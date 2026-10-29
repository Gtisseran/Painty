use eframe::egui::{Color32, Painter, Sense, Shape, Stroke, Ui};

use crate::model::{Brush, Line, Tool};

#[derive(Default)]
pub struct Canvas {
    lines: Vec<Line>,
    current: Option<Line>,
}

impl Canvas {
    pub fn undo(&mut self) {
        self.lines.pop();
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.current = None;
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

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

        // Clic simple sans glisser : on pose un point
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut line = Line::new(brush.color, brush.width, erase);
                line.push(pos);
                line.finish();
                self.lines.push(line);
            }
        }

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
            // Un seul tracé pour tout le trait
            painter.add(Shape::line(pts.clone(), Stroke::new(line.width, color)));

            // Bouts arrondis
            painter.circle_filled(pts[0], radius, color);
            painter.circle_filled(pts[n - 1], radius, color);

            // Jointures arrondies uniquement aux virages serrés
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