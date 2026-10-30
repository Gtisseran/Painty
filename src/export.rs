//! Image export. Draws the strokes at a higher resolution, then averages each block
//! of pixels, which gives smooth edges. Saves PNG or JPEG.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use eframe::egui::{vec2, Color32, Pos2, Rect, Vec2};
use image::codecs::jpeg::JpegEncoder;
use image::{ImageFormat, RgbImage};

use crate::model::Line;

/// Margin kept around the drawing in the exported image, in points.
const MARGIN: f32 = 8.0;
/// The drawing is rendered at this multiple of the output size, then averaged down.
const SUPERSAMPLE: u32 = 3;
/// JPEG quality, from 1 to 100.
const JPEG_QUALITY: u8 = 95;

/// Output file format.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Png,
    Jpeg,
}

impl Format {
    /// File extension, without the dot.
    pub fn ext(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Jpeg => "jpg",
        }
    }

    /// Name shown in the file dialog filter.
    pub fn label(self) -> &'static str {
        match self {
            Format::Png => "PNG image",
            Format::Jpeg => "JPEG image",
        }
    }
}

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

/// Writes the drawing to `path` in the given format.
/// `origin` is the top-left corner of the drawing area and `size` its size, both in points.
/// `ppp` (pixels per point) keeps the export as sharp as the screen.
pub fn save(
    path: &Path,
    format: Format,
    lines: &[Arc<Line>],
    origin: Pos2,
    size: Vec2,
    bg: Color32,
    ppp: f32,
) -> Result<(), String> {
    let w = (size.x * ppp).round().max(1.0) as u32;
    let h = (size.y * ppp).round().max(1.0) as u32;
    let s = SUPERSAMPLE;
    let (bw, bh) = (w * s, h * s);

    // Start with the background color, at the supersampled resolution.
    let mut buf = vec![0u8; (bw as usize) * (bh as usize) * 3];
    for px in buf.chunks_exact_mut(3) {
        px.copy_from_slice(&[bg.r(), bg.g(), bg.b()]);
    }

    {
        let mut raster = Raster {
            buf: &mut buf,
            w: bw,
            h: bh,
        };
        for line in lines {
            raster.draw(line, origin, ppp * s as f32);
        }
    }

    let pixels = downsample(&buf, bw, bh, s);
    let image = RgbImage::from_raw(w, h, pixels).ok_or("invalid image size")?;

    match format {
        Format::Png => image
            .save_with_format(path, ImageFormat::Png)
            .map_err(|e| e.to_string()),
        Format::Jpeg => {
            let file = File::create(path).map_err(|e| e.to_string())?;
            let mut out = BufWriter::new(file);
            JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY)
                .encode_image(&image)
                .map_err(|e| e.to_string())?;
            out.flush().map_err(|e| e.to_string())
        }
    }
}

/// Averages each `s` by `s` block of an RGB buffer of size `sw` by `sh`.
fn downsample(src: &[u8], sw: u32, sh: u32, s: u32) -> Vec<u8> {
    let w = sw / s;
    let h = sh / s;
    let n = s * s;
    let mut out = vec![0u8; (w * h * 3) as usize];

    for y in 0..h {
        for x in 0..w {
            let mut acc = [0u32; 3];
            for dy in 0..s {
                for dx in 0..s {
                    let i = (((y * s + dy) * sw + x * s + dx) * 3) as usize;
                    acc[0] += src[i] as u32;
                    acc[1] += src[i + 1] as u32;
                    acc[2] += src[i + 2] as u32;
                }
            }
            let o = ((y * w + x) * 3) as usize;
            for c in 0..3 {
                out[o + c] = ((acc[c] + n / 2) / n) as u8;
            }
        }
    }
    out
}

/// RGB pixel buffer that strokes are painted into.
struct Raster<'a> {
    buf: &'a mut [u8],
    w: u32,
    h: u32,
}

impl Raster<'_> {
    /// Paints one stroke as a series of round segments.
    fn draw(&mut self, line: &Line, origin: Pos2, ppp: f32) {
        let r = line.width * 0.5 * ppp;
        let pts: Vec<Vec2> = line.points.iter().map(|p| (*p - origin) * ppp).collect();

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
        let x0 = (a.x.min(b.x) - r).floor().max(0.0);
        let x1 = (a.x.max(b.x) + r).ceil().min(self.w as f32 - 1.0);
        let y0 = (a.y.min(b.y) - r).floor().max(0.0);
        let y1 = (a.y.max(b.y) + r).ceil().min(self.h as f32 - 1.0);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let ab = b - a;
        let len2 = ab.length_sq();
        let px = [color.r(), color.g(), color.b()];

        for y in (y0 as u32)..=(y1 as u32) {
            for x in (x0 as u32)..=(x1 as u32) {
                let p = vec2(x as f32 + 0.5, y as f32 + 0.5);
                let t = if len2 > 0.0 {
                    ((p - a).dot(ab) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if (p - (a + ab * t)).length() <= r {
                    let i = ((y * self.w + x) * 3) as usize;
                    self.buf[i..i + 3].copy_from_slice(&px);
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