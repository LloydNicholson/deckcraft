//! Path gradients (`a:path`) that follow a rectangle or the shape's own outline, which no vello
//! gradient draws: they're rasterised into a small image of the bounds and painted with it.
//!
//! Measured on PowerPoint's export (black to grey to white from the centre):
//! - `rect`: the colour position is the larger of the horizontal and vertical distances from the
//!   focus, each as a share of the distance to that side's edge (X-shaped contours);
//! - `shape`: contours are the outline scaled about the focus (an ellipse gives ellipses, a
//!   triangle triangles): the share of the way from the focus to the outline along the same ray.
//!
//! `circle` stays a radial vello gradient (see `paint::gradient`).

use std::sync::Arc;

use deckcraft_color::Rgba;
use kurbo::{Affine, BezPath, PathEl, Point, Rect};
use vello_cpu::{Pixmap, RenderContext, peniko};

/// Longest side of the rasterised gradient, in pixels (bilinear sampling smooths the rest).
const MAX_SIDE: f64 = 384.0;

/// Colour position (0 at the focus, 1 at the edge) of `p` for path `kind` in `bounds` with the
/// focus rect `focus` (fractions inset from left, top, right, bottom), `outline` being the
/// shape's flattened polygon(s) for `shape`.
pub fn position(kind: &str, bounds: Rect, focus: [f64; 4], outline: &[Vec<Point>], p: Point) -> f64 {
    let f = Rect::new(
        bounds.x0 + bounds.width() * focus[0].clamp(0.0, 1.0),
        bounds.y0 + bounds.height() * focus[1].clamp(0.0, 1.0),
        bounds.x1 - bounds.width() * focus[2].clamp(0.0, 1.0),
        bounds.y1 - bounds.height() * focus[3].clamp(0.0, 1.0),
    );
    let f = if f.x0 <= f.x1 && f.y0 <= f.y1 { f } else { Rect::from_center_size(f.center(), (0.0, 0.0)) };
    match kind {
        "shape" if !outline.is_empty() => shape_position(outline, f.center(), p),
        _ => {
            // Distance outside the focus rect along each axis, over the room to that side's edge.
            let axis = |v: f64, lo: f64, hi: f64, edge_lo: f64, edge_hi: f64| {
                if v < lo {
                    (lo - v) / (lo - edge_lo).max(1e-9)
                } else if v > hi {
                    (v - hi) / (edge_hi - hi).max(1e-9)
                } else {
                    0.0
                }
            };
            axis(p.x, f.x0, f.x1, bounds.x0, bounds.x1).max(axis(p.y, f.y0, f.y1, bounds.y0, bounds.y1)).clamp(0.0, 1.0)
        }
    }
}

/// The share of the way from `c` to the outline along the ray through `p` (1 when the ray meets
/// no edge).
fn shape_position(outline: &[Vec<Point>], c: Point, p: Point) -> f64 {
    let d = p - c;
    let len = d.hypot();
    if len < 1e-9 {
        return 0.0;
    }
    // Nearest crossing of the ray c + s·d (s > 0) with an edge.
    let mut best = f64::INFINITY;
    for poly in outline {
        for w in poly.windows(2) {
            let (a, b) = (w[0], w[1]);
            let e = b - a;
            let den = d.cross(e);
            if den.abs() < 1e-12 {
                continue;
            }
            let ac = a - c;
            let s = ac.cross(e) / den;
            let u = ac.cross(d) / den;
            if s > 1e-9 && (-1e-9..=1.0 + 1e-9).contains(&u) {
                best = best.min(s);
            }
        }
    }
    if best.is_finite() { (1.0 / best).clamp(0.0, 1.0) } else { 1.0 }
}

/// The outline flattened into closed polygons.
pub fn polygons(path: &BezPath) -> Vec<Vec<Point>> {
    let mut out: Vec<Vec<Point>> = Vec::new();
    let mut cur: Vec<Point> = Vec::new();
    kurbo::flatten(path.iter(), 0.25, |el| match el {
        PathEl::MoveTo(p) => {
            if cur.len() > 1 {
                out.push(std::mem::take(&mut cur));
            }
            cur = vec![p];
        }
        PathEl::LineTo(p) => cur.push(p),
        PathEl::ClosePath => {
            if let Some(f) = cur.first().copied() {
                cur.push(f);
            }
        }
        _ => {}
    });
    if cur.len() > 1 {
        out.push(cur);
    }
    for poly in &mut out {
        if let (Some(f), Some(l)) = (poly.first().copied(), poly.last().copied())
            && (f - l).hypot() > 1e-9
        {
            poly.push(f);
        }
    }
    out.retain(|p| p.len() > 2 && p.len() < 100_000);
    out
}

/// A 256-entry colour ramp for the sorted, already expanded `stops` (linear in sRGB between them).
pub fn ramp(stops: &[(f64, Rgba)]) -> Vec<Rgba> {
    (0..256)
        .map(|i| {
            let t = i as f64 / 255.0;
            let after = stops.iter().position(|s| s.0 >= t);
            match after {
                Some(0) | None => stops.get(after.unwrap_or(stops.len().saturating_sub(1))).map(|s| s.1).unwrap_or(Rgba::BLACK),
                Some(j) => {
                    let (Some(a), Some(b)) = (stops.get(j - 1), stops.get(j)) else { return Rgba::BLACK };
                    let k = if b.0 - a.0 > 1e-12 { (t - a.0) / (b.0 - a.0) } else { 1.0 };
                    a.1.lerp(b.1, k)
                }
            }
        })
        .collect()
}

/// Fill `path` with a `rect` or `shape` path gradient of `stops` (sorted and expanded, see
/// `ramp::expand`) over `bounds`, faded by `alpha`; `m` maps the path to the output.
#[allow(clippy::too_many_arguments)]
pub fn fill(ctx: &mut RenderContext, kind: &str, focus: [f64; 4], stops: &[(f64, Rgba)], bounds: Rect, path: &BezPath, m: Affine, alpha: f64) {
    if bounds.width() <= 0.0 || bounds.height() <= 0.0 || !bounds.is_finite() || stops.is_empty() {
        return;
    }
    let k = crate::scale_of(m);
    let k = if k.is_finite() && k > 0.0 { k } else { 1.0 };
    let longest = bounds.width().max(bounds.height()) * k;
    let s = (MAX_SIDE / longest).min(1.0) * k;
    let (pw, ph) = (((bounds.width() * s).ceil() as u32).clamp(1, 4096), ((bounds.height() * s).ceil() as u32).clamp(1, 4096));
    let outline = if kind == "shape" { polygons(path) } else { Vec::new() };
    let lut = ramp(stops);
    let a = alpha.clamp(0.0, 1.0);
    let mut data = Vec::with_capacity((pw * ph) as usize);
    for j in 0..ph {
        for i in 0..pw {
            let p = Point::new(bounds.x0 + (i as f64 + 0.5) / pw as f64 * bounds.width(), bounds.y0 + (j as f64 + 0.5) / ph as f64 * bounds.height());
            let t = position(kind, bounds, focus, &outline, p);
            let c = lut.get((t * 255.0).round().clamp(0.0, 255.0) as usize).copied().unwrap_or(Rgba::BLACK);
            let ca = (c.a as f64 * a).round().clamp(0.0, 255.0);
            let mul = |v: u8| (v as f64 * ca / 255.0).round().clamp(0.0, 255.0) as u8;
            data.push(vello_cpu::color::PremulRgba8 { r: mul(c.r), g: mul(c.g), b: mul(c.b), a: ca as u8 });
        }
    }
    let pm = Arc::new(Pixmap::from_parts(data, pw as u16, ph as u16));
    ctx.set_transform(m);
    ctx.set_paint(vello_cpu::Image {
        image: vello_cpu::ImageSource::Pixmap(pm),
        sampler: peniko::ImageSampler { quality: peniko::ImageQuality::Medium, ..Default::default() },
    });
    ctx.set_paint_transform(
        Affine::translate(bounds.origin().to_vec2()) * Affine::scale_non_uniform(bounds.width() / pw as f64, bounds.height() / ph as f64),
    );
    ctx.fill_path(path);
    ctx.reset_paint_transform();
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;

    const CENTRE: [f64; 4] = [0.5, 0.5, 0.5, 0.5];

    #[test]
    fn rect_paths_take_the_larger_axis_share() {
        // PowerPoint, 280x200 box from the centre: 20 px across is 1/7, 20 px down is 1/5.
        let b = Rect::new(0.0, 0.0, 280.0, 200.0);
        let at = |x: f64, y: f64| position("rect", b, CENTRE, &[], Point::new(x, y));
        assert!((at(160.0, 100.0) - 20.0 / 140.0).abs() < 1e-9);
        assert!((at(140.0, 120.0) - 20.0 / 100.0).abs() < 1e-9);
        assert!((at(168.0, 120.0) - 0.2).abs() < 1e-9, "the diagonal takes the larger share");
        assert_eq!(at(0.0, 0.0), 1.0);
        // From a corner (fillToRect 0,0,100%,100%): the far corner is the end.
        let corner = [0.0, 0.0, 1.0, 1.0];
        assert!((position("rect", b, corner, &[], Point::new(140.0, 50.0)) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn shape_paths_follow_the_outline() {
        let b = Rect::new(0.0, 0.0, 280.0, 200.0);
        let ellipse = polygons(&kurbo::Ellipse::from_rect(b).to_path(0.01));
        let at = |x: f64, y: f64| position("shape", b, CENTRE, &ellipse, Point::new(x, y));
        assert!((at(160.0, 100.0) - 20.0 / 140.0).abs() < 0.01);
        assert!((at(140.0, 120.0) - 0.2).abs() < 0.01);
        // Half way to the outline along any ray is 0.5.
        let (dx, dy) = (140.0 * 0.6, 100.0 * 0.8);
        assert!((at(140.0 + dx * 0.5, 100.0 + dy * 0.5) - 0.5).abs() < 0.01);
        // A triangle: half way from its centre to its apex is 0.5.
        let mut tri = BezPath::new();
        tri.move_to((140.0, 0.0));
        tri.line_to((280.0, 200.0));
        tri.line_to((0.0, 200.0));
        tri.close_path();
        let polys = polygons(&tri);
        assert!((position("shape", b, CENTRE, &polys, Point::new(140.0, 50.0)) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn the_ramp_runs_through_the_stops() {
        let stops = [(0.0, Rgba::rgb(0, 0, 0)), (0.5, Rgba::rgb(128, 128, 128)), (1.0, Rgba::rgb(255, 255, 255))];
        let r = ramp(&stops);
        assert_eq!(r.len(), 256);
        assert_eq!(r[0], Rgba::rgb(0, 0, 0));
        assert_eq!(r[255], Rgba::rgb(255, 255, 255));
        assert!((r[64].r as i32 - 64).abs() <= 2);
        assert!(ramp(&[(0.3, Rgba::rgb(9, 9, 9))]).iter().all(|c| *c == Rgba::rgb(9, 9, 9)));
    }
}
