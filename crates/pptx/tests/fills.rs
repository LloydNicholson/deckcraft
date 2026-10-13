//! Fills read from decks written by other tools.

use std::io::{Cursor, Read, Write};

use deckcraft_geom::Xfrm;
use deckcraft_model::style::{Fill, PictureFill, PictureMode};
use deckcraft_model::{Shape, ShapeId, defaults};

/// `bytes` with `edit` applied to every slide part.
fn edit_slides(bytes: &[u8], edit: impl Fn(String) -> String) -> Vec<u8> {
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let o = zip::write::SimpleFileOptions::default();
    for i in 0..z.len() {
        let mut f = z.by_index(i).expect("entry");
        let mut d = vec![];
        f.read_to_end(&mut d).expect("read");
        let name = f.name().to_string();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            d = edit(String::from_utf8(d).expect("utf8")).into_bytes();
        }
        out.start_file(name, o).expect("start");
        out.write_all(&d).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

fn png() -> Vec<u8> {
    let mut out = Cursor::new(vec![]);
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 4, image::Rgb([10, 200, 30])))
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("png");
    out.into_inner()
}

/// A deck with one shape filled with a stretched picture, exported.
fn deck() -> Vec<u8> {
    let mut p = defaults::new_presentation(None);
    let media = p.add_media("tile.png", "image/png", png());
    let id = ShapeId(p.alloc_id());
    let first = p.slides[0].id;
    p.slide_mut(first).expect("slide").shapes.push(Shape {
        id,
        name: "Textured".into(),
        xfrm: Some(Xfrm::new(10.0, 10.0, 200.0, 100.0)),
        fill: Some(Fill::Picture(PictureFill { media, ..Default::default() })),
        ..Default::default()
    });
    deckcraft_pptx::export(&p).expect("export")
}

fn textured_mode(bytes: &[u8]) -> PictureMode {
    let q = deckcraft_pptx::import(bytes).expect("import");
    let s = q.slides[0].shapes.iter().find(|s| s.name == "Textured").expect("shape");
    match &s.fill {
        Some(Fill::Picture(pf)) => pf.mode.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_shape_picture_fill_without_stretch_or_tile_tiles() {
    // tdf#128596: `<a:blipFill><a:blip/></a:blipFill>` on a shape, as some tools write it;
    // PowerPoint tiles the picture at its own size.
    let bare = edit_slides(&deck(), |s| {
        let s = regex_free_strip(&s, "<a:stretch>", "</a:stretch>");
        assert!(!s.contains("<a:stretch"), "the test deck lost its stretch");
        s
    });
    assert!(matches!(textured_mode(&bare), PictureMode::Tile { sx, sy, .. } if sx == 1.0 && sy == 1.0));
    // An explicit stretch is still a stretch.
    assert!(matches!(textured_mode(&deck()), PictureMode::Stretch { .. }));
}

/// `s` without the first `open`…`close` span inside a `blipFill`.
fn regex_free_strip(s: &str, open: &str, close: &str) -> String {
    let Some(b) = s.find("<a:blipFill") else { return s.to_string() };
    let Some(i) = s[b..].find(open).map(|i| i + b) else { return s.to_string() };
    let Some(j) = s[i..].find(close).map(|j| j + i + close.len()) else { return s.to_string() };
    format!("{}{}", &s[..i], &s[j..])
}

#[test]
fn a_gradient_without_stops_is_saved_without_stops() {
    // `a:gradFill` with no `a:gsLst` takes its stops from the style's fill in PowerPoint; saving
    // must not turn it into a gradient of its own (it used to become solid white).
    use deckcraft_model::style::{Gradient, GradientShape};
    let mut p = defaults::new_presentation(None);
    let id = ShapeId(p.alloc_id());
    let first = p.slides[0].id;
    let own = Gradient { stops: vec![], shape: GradientShape::Linear { angle: 45.0, scaled: true }, rotate_with_shape: true };
    p.slide_mut(first).expect("slide").shapes.push(Shape {
        id,
        name: "Styled".into(),
        xfrm: Some(Xfrm::new(10.0, 10.0, 200.0, 100.0)),
        fill: Some(Fill::Gradient(own.clone())),
        style: Some(deckcraft_model::ShapeStyle::accent(deckcraft_color::SchemeSlot::Accent1)),
        ..Default::default()
    });
    let bytes = deckcraft_pptx::export(&p).expect("export");
    let q = deckcraft_pptx::import(&bytes).expect("import");
    let s = q.slides[0].shapes.iter().find(|s| s.name == "Styled").expect("shape");
    assert_eq!(s.fill, Some(Fill::Gradient(own)));
}

#[test]
fn a_fill_overlay_round_trips() {
    use deckcraft_model::style::{FillOverlay, Gradient, GradientShape, GradientStop};
    let mut p = defaults::new_presentation(None);
    let id = ShapeId(p.alloc_id());
    let first = p.slides[0].id;
    let overlay = FillOverlay {
        blend: "mult".into(),
        fill: Fill::Gradient(Gradient {
            stops: vec![
                GradientStop { pos: 0.0, color: deckcraft_model::ColorRef::scheme(deckcraft_color::SchemeSlot::Accent1) },
                GradientStop { pos: 1.0, color: deckcraft_model::ColorRef::rgb(deckcraft_color::Rgba::rgb(10, 20, 30)) },
            ],
            shape: GradientShape::Linear { angle: 4.5, scaled: true },
            rotate_with_shape: true,
        }),
    };
    p.slide_mut(first).expect("slide").shapes.push(Shape {
        id,
        name: "Overlaid".into(),
        xfrm: Some(Xfrm::new(10.0, 10.0, 200.0, 100.0)),
        fill: Some(Fill::solid(deckcraft_model::ColorRef::rgb(deckcraft_color::Rgba::WHITE))),
        effects: Some(deckcraft_model::Effects { fill_overlay: Some(overlay.clone()), ..Default::default() }),
        ..Default::default()
    });
    let bytes = deckcraft_pptx::export(&p).expect("export");
    let q = deckcraft_pptx::import(&bytes).expect("import");
    let s = q.slides[0].shapes.iter().find(|s| s.name == "Overlaid").expect("shape");
    assert_eq!(s.effects.as_ref().and_then(|e| e.fill_overlay.clone()), Some(overlay));
}
