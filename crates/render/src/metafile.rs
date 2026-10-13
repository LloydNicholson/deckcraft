//! Windows Metafile and Enhanced Metafile pictures (`image/x-wmf`, `image/x-emf`): parsed by
//! `deckcraft-metafile` and drawn as vectors into a bitmap of the size the slide needs, so they go
//! through the same crop, tile and recolour path as any other picture.

use std::sync::Arc;

use deckcraft_metafile::PlacedItem;
use kurbo::{Affine, Rect, Shape as _};
use vello_cpu::{Pixmap, RenderContext, Resources, peniko};

/// Longest side of a metafile drawn without a target size (export, thumbnails of the picture).
pub const DEFAULT_SIDE: u32 = 1024;
/// Largest side drawn.
const MAX_SIDE: u32 = 8192;

/// Is `bytes` a WMF or EMF file?
pub fn is_metafile(bytes: &[u8]) -> bool {
    deckcraft_metafile::is_metafile(bytes)
}

/// Draw the metafile `bytes` into straight RGBA with its longer side `side` px (transparent where
/// it draws nothing). `None` when it doesn't parse or has no size.
pub fn rasterize(bytes: &[u8], side: u32) -> Option<image::RgbaImage> {
    let mut pic = deckcraft_metafile::parse(bytes).ok()?;
    if !pic.has_size() {
        return None;
    }
    let side = side.clamp(16, MAX_SIDE) as f64;
    let (pw, ph) = (pic.width as f64, pic.height as f64);
    let k = side / pw.max(ph);
    let w = (pw * k).round().clamp(1.0, MAX_SIDE as f64) as u16;
    let h = (ph * k).round().clamp(1.0, MAX_SIDE as f64) as u16;
    let pixmaps: Vec<Option<Arc<Pixmap>>> = pic.items.iter_mut().map(|it| it.take_pixels().and_then(|(w, h, px)| premul(w, h, &px))).collect();
    let mut ctx = RenderContext::new_with(w, h, vello_cpu::RenderSettings { num_threads: 0, ..Default::default() });
    let r = Rect::new(0.0, 0.0, w as f64, h as f64);
    ctx.push_clip_layer(&r.to_path(0.1));
    for it in pic.place(r, [0.0; 4]) {
        match it {
            PlacedItem::Path { path, fill, stroke, even_odd } => {
                if let Some(f) = fill {
                    ctx.set_paint(rgba(f));
                    ctx.set_fill_rule(if even_odd { peniko::Fill::EvenOdd } else { peniko::Fill::NonZero });
                    ctx.fill_path(&path);
                }
                // GDI pens: round caps and joins; a cosmetic (zero-width) pen is one pixel.
                if let Some((s, sw)) = stroke {
                    ctx.set_paint(rgba(s));
                    ctx.set_stroke(kurbo::Stroke::new(sw.max(1.0)).with_caps(kurbo::Cap::Round).with_join(kurbo::Join::Round));
                    ctx.stroke_path(&path);
                }
            }
            PlacedItem::Bitmap { index, rect } => {
                let Some(Some(pm)) = pixmaps.get(index) else { continue };
                let (bw, bh) = (pm.width().max(1) as f64, pm.height().max(1) as f64);
                ctx.set_paint(vello_cpu::Image {
                    image: vello_cpu::ImageSource::Pixmap(pm.clone()),
                    sampler: peniko::ImageSampler::default().with_quality(peniko::ImageQuality::Medium),
                });
                ctx.set_paint_transform(Affine::translate((rect.x0, rect.y0)) * Affine::scale_non_uniform(rect.width() / bw, rect.height() / bh));
                ctx.fill_rect(&rect);
                ctx.reset_paint_transform();
            }
        }
    }
    ctx.set_fill_rule(peniko::Fill::NonZero);
    ctx.pop_layer();
    ctx.flush();
    let mut px = vec![0u8; w as usize * h as usize * 4];
    if let Some(pm) = vello_cpu::PixmapMut::new(w, h, &mut px) {
        ctx.render(pm, &mut Resources::new());
    }
    // Premultiplied to straight.
    for p in px.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        if a != 0 && a != 255 {
            for c in p.iter_mut().take(3) {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    image::RgbaImage::from_raw(w as u32, h as u32, px)
}

fn rgba(c: [u8; 4]) -> peniko::Color {
    peniko::Color::from_rgba8(c[0], c[1], c[2], c[3])
}

/// Straight RGBA rows to a premultiplied pixmap.
fn premul(w: u32, h: u32, px: &[u8]) -> Option<Arc<Pixmap>> {
    if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 || px.len() < w as usize * h as usize * 4 {
        return None;
    }
    let data = px
        .as_chunks::<4>()
        .0
        .iter()
        .take(w as usize * h as usize)
        .map(|p| {
            let a = p[3] as u16;
            let m = |c: u8| ((c as u16 * a + 127) / 255) as u8;
            vello_cpu::color::PremulRgba8 { r: m(p[0]), g: m(p[1]), b: m(p[2]), a: p[3] }
        })
        .collect();
    Some(Arc::new(Pixmap::from_parts(data, w as u16, h as u16)))
}
