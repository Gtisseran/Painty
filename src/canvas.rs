use std::sync::Arc;

use eframe::egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui};

use crate::model::{Brush, Line, Tool};

/// Maximum number of undo steps kept in memory.
const HISTORY_LIMIT: usize = 100;
/// How much the drawn point lags behind the pointer, from 0.0 (none) to 1.0 (very smooth).
/// Lower it for a more direct feel, raise it for smoother strokes.
const SMOOTHING: f32 = 0.6;
/// Eraser step, as a fraction of the eraser radius. Smaller = no gaps, slower.
const ERASE_STEP: f32 = 0.25;

/// Strokes are shared with `Arc`, so a history snapshot only copies pointers.
type Lines = Vec<Arc<Line>>;

/// Holds the finished strokes, the stroke being drawn, and the undo/redo history.
#[derive(Default)]
pub struct Canvas {
    lines: Lines,
    current: Option<Line>,
    undo: Vec<Lines>,
    redo: Vec<Lines>,
    /// Smoothed pointer position used while drawing or erasing.
    smooth_pos: Option<Pos2>,
    /// Last erased position during a drag, so fast drags leave no gaps.
    last_erase: Option<Pos2>,
    /// Area of the drawing area on screen, used when exporting.
    rect: Option<Rect>,
}

impl Canvas {
    /// Goes back one step.
    pub fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            let current = std::mem::replace(&mut self.lines, previous);
            self.redo.push(current);
        }
    }

    /// Goes forward one step, after an undo.
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            let current = std::mem::replace(&mut self.lines, next);
            self.undo.push(current);
        }
    }

    /// Removes everything. This cannot be undone.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current = None;
        self.undo.clear();
        self.redo.clear();
        self.smooth_pos = None;
        self.last_erase = None;
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// All finished strokes, for export.
    pub fn lines(&self) -> &[Arc<Line>] {
        &self.lines
    }

    /// Area of the drawing area on screen, once it has been drawn.
    pub fn rect(&self) -> Option<Rect> {
        self.rect
    }

    /// Handles mouse input on the drawing area and paints all strokes.
    /// `ring` is the color of the eraser outline.
    pub fn show(&mut self, ui: &mut Ui, brush: &Brush, ring: Color32) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.rect = Some(response.rect);
        let radius = brush.width / 2.0;
        let erasing = brush.tool == Tool::Eraser;

        if erasing {
            self.handle_eraser(&response, radius);
        } else {
            self.handle_brush(&response, brush);
        }

        // Only strokes that touch the visible area are drawn.
        let clip = painter.clip_rect();
        for line in &self.lines {
            if line.bounds.intersects(clip) {
                draw_line(&painter, line);
            }
        }
        if let Some(line) = &self.current {
            draw_line(&painter, line);
        }

        // Outline of the eraser under the pointer.
        if erasing {
            if let Some(pos) = response.hover_pos() {
                painter.circle_stroke(pos, radius, Stroke::new(1.0_f32, ring));
            }
        }
    }

    /// Moves the smoothed position toward the pointer and returns it.
    fn smooth(&mut self, target: Pos2) -> Pos2 {
        let next = match self.smooth_pos {
            Some(prev) => prev + (target - prev) * (1.0 - SMOOTHING),
            None => target,
        };
        self.smooth_pos = Some(next);
        next
    }

    /// Saves the current state so it can be restored with undo.
    fn checkpoint(&mut self) {
        self.undo.push(self.lines.clone());
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Adds a finished stroke, as one undoable step.
    fn commit(&mut self, line: Line) {
        self.checkpoint();
        self.lines.push(Arc::new(line));
    }

    /// Brush: draws a smooth stroke while dragging, or a single dot on click.
    fn handle_brush(&mut self, response: &Response, brush: &Brush) {
        if response.drag_started() {
            self.current = Some(Line::new(brush.color, brush.width));
            self.smooth_pos = response.interact_pointer_pos();
        }

        if response.dragged() {
            if let Some(target) = response.interact_pointer_pos() {
                let p = self.smooth(target);
                if let Some(line) = self.current.as_mut() {
                    line.push(p);
                }
            }
        }

        if response.drag_stopped() {
            if let Some(mut line) = self.current.take() {
                // Finish exactly where the stroke was heading, so it does not stop short.
                if let Some(end) = self.smooth_pos {
                    line.push(end);
                }
                if !line.points.is_empty() {
                    line.finish();
                    self.commit(line);
                }
            }
            self.smooth_pos = None;
        }

        // Single click without dragging: place one point.
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut line = Line::new(brush.color, brush.width);
                line.push(pos);
                line.finish();
                self.commit(line);
            }
        }
    }

    /// Eraser: removes the parts of existing strokes under the pointer, with smoothing.
    fn handle_eraser(&mut self, response: &Response, radius: f32) {
        if response.drag_started() || response.clicked() {
            self.checkpoint();
            self.last_erase = None;
            self.smooth_pos = response.interact_pointer_pos();
        }

        if response.dragged() {
            if let Some(target) = response.interact_pointer_pos() {
                let p = self.smooth(target);
                let from = self.last_erase.unwrap_or(p);
                self.erase_between(from, p, radius);
                self.last_erase = Some(p);
            }
        }

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.erase_at(pos, radius);
            }
        }

        if response.drag_stopped() {
            self.last_erase = None;
            self.smooth_pos = None;
        }
    }

    /// Erases along the segment from `from` to `to`, in small steps.
    fn erase_between(&mut self, from: Pos2, to: Pos2, radius: f32) {
        let step = (radius * ERASE_STEP).max(0.5);
        let steps = (from.distance(to) / step).ceil().max(1.0) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.erase_at(from + (to - from) * t, radius);
        }
    }

    /// Erases every point within `radius` of `p`. Strokes are split where the eraser passes.
    fn erase_at(&mut self, p: Pos2, radius: f32) {
        let mut next: Lines = Vec::with_capacity(self.lines.len());
        let mut changed = false;

        for line in self.lines.iter() {
            let reach = radius + line.width * 0.5;
            if !line.bounds.expand(reach).contains(p) {
                next.push(line.clone());
                continue;
            }

            let keep: Vec<bool> = line
                .points
                .iter()
                .map(|q| q.distance(p) > reach)
                .collect();
            if keep.iter().all(|&k| k) {
                next.push(line.clone());
                continue;
            }

            changed = true;
            // Split the stroke into runs of points that were not erased.
            let mut run: Vec<Pos2> = Vec::new();
            for (q, &kept) in line.points.iter().zip(&keep) {
                if kept {
                    run.push(*q);
                } else if !run.is_empty() {
                    next.push(Arc::new(Line::from_points(
                        line.color,
                        line.width,
                        std::mem::take(&mut run),
                    )));
                }
            }
            if !run.is_empty() {
                next.push(Arc::new(Line::from_points(line.color, line.width, run)));
            }
        }

        if changed {
            self.lines = next;
        }
    }
}

/// Draws one stroke as a polyline with round ends and round sharp corners.
fn draw_line(painter: &Painter, line: &Line) {
    let color = line.color;
    let radius = line.width / 2.0;
    let pts = &line.points;

    match pts.len() {
        0 => {}
        1 => {
            painter.circle_filled(pts[0], radius, color);
        }
        n => {
            // Draw the whole stroke as one polyline.
            painter.add(Shape::line(pts.clone(), Stroke::new(line.width, color)));

            // Round caps at both ends.
            painter.circle_filled(pts[0], radius, color);
            painter.circle_filled(pts[n - 1], radius, color);

            // Round joins only at sharp corners (dot product of the two segments below 0.8).
            for w in pts.windows(3) {
                let a = w[1] - w[0];
                let b = w[2] - w[1];
                let (la, lb) = (a.length(), b.length());
                if la > 0.0 && lb > 0.0 && a.dot(b) / (la * lb) < 0.8 {
                    painter.circle_filled(w[1], radius, color);
                }
            }
        }
    }
}