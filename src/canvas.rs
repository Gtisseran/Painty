use eframe::egui::{Color32, Painter, Sense, Shape, Stroke, Ui};

use crate::model::{Brush, Line, Tool};

/// Holds the finished strokes and the stroke currently being drawn.
#[derive(Default)]
pub struct Canvas {
    lines: Vec<Line>,
    current: Option<Line>,
}

impl Canvas {
    /// Removes the last finished stroke.
    pub fn undo(&mut self) {
        self.lines.pop();
    }

    /// Removes every stroke.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current = None;
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Handles mouse input on the drawing area and paints all strokes.
    pub fn show(&mut self, ui: &mut Ui, brush: &Brush, bg: Color32) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let erase = brush.tool == Tool::Eraser;

        if response.drag_started() {
            self.current = Some(Line::new(brush.color, brush.width, erase));
        }

        if response.dragged() {
            if let (Some(pos), Some(line)) =
                (response.interact_pointer_pos(), self.current.as_mut())
            {
                line.push(pos);
            }
        }

        if response.drag_stopped() {
            if let Some(mut line) = self.current.take() {
                if !line.points.is_empty() {
                    line.finish();
                    self.lines.push(line);
                }
            }
        }

        // Single click without dragging: place one point.
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut line = Line::new(brush.color, brush.width, erase);
                line.push(pos);
                line.finish();
                self.lines.push(line);
            }
        }

        // Skip strokes that are outside the visible area.
        let clip = painter.clip_rect();
        for line in &self.lines {
            if line.bounds.intersects(clip) {
                draw_line(&painter, line, bg);
            }
        }
        if let Some(line) = &self.current {
            draw_line(&painter, line, bg);
        }
    }
}

/// Draws one stroke. Eraser strokes are painted with the background color.
fn draw_line(painter: &Painter, line: &Line, bg: Color32) {
    let color = if line.erase { bg } else { line.color };
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