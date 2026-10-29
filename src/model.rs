use eframe::egui::{Color32, Pos2, Rect};

use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Brush,
    Eraser,
}

pub struct Line {
    pub points: Vec<Pos2>,
    pub color: Color32,
    pub width: f32,
    pub erase: bool,
    pub bounds: Rect,
}

impl Line {
    pub fn new(color: Color32, width: f32, erase: bool) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
            erase,
            bounds: Rect::NOTHING,
        }
    }

    /// Ajoute un point en ignorant ceux trop proches du précédent.
    pub fn push(&mut self, p: Pos2) {
        let min_step = (self.width * 0.1).max(1.0);
        if let Some(last) = self.points.last() {
            if last.distance(p) < min_step {
                return;
            }
        }
        self.points.push(p);
    }

    /// Calcule la zone occupée (utilisée pour ne pas dessiner hors écran).
    pub fn finish(&mut self) {
        if !self.points.is_empty() {
            self.bounds = Rect::from_points(&self.points).expand(self.width / 2.0 + 1.0);
        }
    }
}

pub struct Brush {
    pub color: Color32,
    pub width: f32,
    pub tool: Tool,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            color: theme::INK_LIGHT,
            width: 6.0,
            tool: Tool::Brush,
        }
    }
}