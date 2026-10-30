//! PNG export. Draws the strokes into an in-memory image, then saves it.

use std::path::Path;
use std::sync::Arc;

use eframe::egui::{vec2, Color32, Pos2, Rect, Vec2};

use crate::model::Line;

/// Margin kept around the drawing in the exported image, in points.
const MARGIN: f32 = 8.0;

/// Bounding box of all strokes, with a margin, in screen points.
/// Returns `None` when there is nothing to export.
pub fn bounds(lines: &[Arc<Line>]) -> Option<Rect> {
    let mut bounds = Rect::NOTHING;
    for line in lines {
        bounds = bounds.union(line.bounds);
    }
    if bounds.is_positive() {
        Some(bounds.expand(MARGIN))
    } else {
        None
    }
}

/// Writes the drawing to `path` as a PNG.
/// `origin` is the top-left corner of the drawing area and `size` its size, both in points.
/// `ppp` (pixels per point) keeps the export as sharp as the screen.
pub fn save_png(
    path: &Path,
    lines: &[Arc<Line>],
    origin: Pos2,
    size: Vec2,
    bg: Color32,
    ppp: f32,
) -> Result<(), String> {
    let w = (size.x * ppp).round().max(1.0) as u32;
    let h = (size.y * ppp).round().max(1.0) as u32;

    // Start with the background color, fully opaque.
    let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
    for px in buf.chunks_exact_mut(4) {
        px.copy_from_slice(&[bg.r(), bg.g(), bg.b(), 255]);
    }

    {
        let mut raster = Raster { buf: &mut buf, w, h };
        for line in lines {
            raster.draw(line, origin, ppp);
        }
    }

    let image = image::RgbaImage::from_raw(w, h, buf).ok_or("invalid image size")?;
    image.save(path).map_err(|e| e.to_string())
}

/// Pixel buffer that strokes are painted into.
struct Raster<'a> {
    buf: &'a mut [u8],
    w: u32,
    h: u32,
}

impl Raster<'_> {
    /// Paints one stroke as a series of round segments.
    fn draw(&mut self, line: &Line, origin: Pos2, ppp: f32) {
        let r = line.width * 0.5 * ppp;
        let pts: Vec<Vec2> = line
            .points
            .iter()
            .map(|p| (*p - origin) * ppp)
            .collect();

        match pts.len() {
            0 => {}
            1 => self.segment(pts[0], pts[0], r, line.color),
            _ => {
                for pair in pts.windows(2) {
                    self.segment(pair[0], pair[1], r, line.color);
                }
            }
        }
    }

    /// Fills every pixel within `r` of the segment from `a` to `b`.
    fn segment(&mut self, a: Vec2, b: Vec2, r: f32, color: Color32) {
        // Bounding box of the segment, clamped to the image.
        let x0 = (a.x.min(b.x) - r).floor().max(0.0);
        let x1 = (a.x.max(b.x) + r).ceil().min(self.w as f32 - 1.0);
        let y0 = (a.y.min(b.y) - r).floor().max(0.0);
        let y1 = (a.y.max(b.y) + r).ceil().min(self.h as f32 - 1.0);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let ab = b - a;
        let len2 = ab.length_sq();
        let px = [color.r(), color.g(), color.b(), 255];

        for y in (y0 as u32)..=(y1 as u32) {
            for x in (x0 as u32)..=(x1 as u32) {
                let p = vec2(x as f32 + 0.5, y as f32 + 0.5);
                // Closest point of the segment to this pixel.
                let t = if len2 > 0.0 {
                    ((p - a).dot(ab) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if (p - (a + ab * t)).length() <= r {
                    let i = ((y * self.w + x) * 4) as usize;
                    self.buf[i..i + 4].copy_from_slice(&px);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::pos2;

    fn stroke(points: &[Pos2]) -> Arc<Line> {
        Arc::new(Line::from_points(Color32::BLACK, 4.0, points.to_vec()))
    }

    #[test]
    fn bounds_cover_every_stroke() {
        let lines = vec![
            stroke(&[pos2(10.0, 10.0)]),
            stroke(&[pos2(100.0, 50.0), pos2(200.0, 80.0)]),
        ];
        let expected = lines[0].bounds.union(lines[1].bounds).expand(MARGIN);
        assert_eq!(bounds(&lines), Some(expected));
    }

    #[test]
    fn empty_drawing_has_no_bounds() {
        assert_eq!(bounds(&[]), None);
    }
}