//! The native `.deckcraft` format: a zip holding `presentation.json` (the model, minus media
//! bytes), `media/<id>` files with the embedded media, and `mimetype`. Readers ignore unknown
//! fields, so newer files open in older builds with what those builds understand.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::io::{Cursor, Read, Write};
use std::sync::Arc;

use deckcraft_model::text::{Paragraph, TextBody};
use deckcraft_model::{LayoutType, Presentation, defaults};

pub const MIMETYPE: &str = "application/vnd.storyteller.deckcraft+zip";
/// Files written before the SlideCraft → DeckCraft rename (`.slidecraft`) are still read.
pub const LEGACY_MIMETYPE: &str = concat!("application/vnd.storyteller.", "slide", "craft+zip");
pub const EXTENSION: &str = "deckcraft";
/// Format version written into the manifest.
pub const VERSION: u32 = 1;
/// Largest entry we will inflate (guards against zip bombs).
const MAX_ENTRY: u64 = 2 << 30;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("not a DeckCraft file: {0}")]
    NotOurs(String),
    #[error("damaged file: {0}")]
    Damaged(String),
    #[error("write failed: {0}")]
    Write(String),
}

pub type Result<T> = std::result::Result<T, FormatError>;

#[derive(serde::Serialize, serde::Deserialize)]
struct Manifest {
    format: String,
    version: u32,
    generator: String,
    presentation: Presentation,
}

/// Serialize a presentation.
pub fn save(p: &Presentation) -> Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let deflate = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let w = |e: zip::result::ZipError| FormatError::Write(e.to_string());
        z.start_file("mimetype", stored).map_err(w)?;
        z.write_all(MIMETYPE.as_bytes()).map_err(|e| FormatError::Write(e.to_string()))?;
        let m = Manifest {
            format: "deckcraft".into(),
            version: VERSION,
            generator: format!("DeckCraft {}", env!("CARGO_PKG_VERSION")),
            presentation: p.clone(),
        };
        let json = serde_json::to_vec(&m).map_err(|e| FormatError::Write(e.to_string()))?;
        z.start_file("presentation.json", deflate).map_err(w)?;
        z.write_all(&json).map_err(|e| FormatError::Write(e.to_string()))?;
        for item in &p.media {
            if item.link.is_some() && item.data.is_empty() {
                continue;
            }
            // Already-compressed media is stored, not deflated again.
            let opts = if is_compressed(&item.content_type) { stored } else { deflate };
            z.start_file(format!("media/{}", item.id.0), opts).map_err(w)?;
            z.write_all(&item.data).map_err(|e| FormatError::Write(e.to_string()))?;
        }
        z.finish().map_err(w)?;
    }
    Ok(buf.into_inner())
}

fn is_compressed(ct: &str) -> bool {
    ct.starts_with("image/jpeg")
        || ct.starts_with("image/png")
        || ct.starts_with("video/")
        || ct.starts_with("audio/mpeg")
        || ct.starts_with("audio/mp4")
        || ct.contains("zip")
}

/// Does this look like a `.deckcraft` file?
pub fn sniff(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK") && [MIMETYPE, LEGACY_MIMETYPE].iter().any(|m| bytes.windows(m.len()).take(200).any(|w| w == m.as_bytes()))
}

fn read_entry(z: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<Vec<u8>> {
    let f = z.by_name(name).map_err(|e| FormatError::Damaged(format!("{name}: {e}")))?;
    if f.size() > MAX_ENTRY {
        return Err(FormatError::Damaged(format!("{name} is too large")));
    }
    let mut out = Vec::with_capacity(f.size().min(64 << 20) as usize);
    f.take(MAX_ENTRY).read_to_end(&mut out).map_err(|e| FormatError::Damaged(format!("{name}: {e}")))?;
    Ok(out)
}

/// Read a presentation.
pub fn load(bytes: &[u8]) -> Result<Presentation> {
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    let json = read_entry(&mut z, "presentation.json").map_err(|_| FormatError::NotOurs("no presentation.json".into()))?;
    let m: Manifest = serde_json::from_slice(&json).map_err(|e| FormatError::Damaged(e.to_string()))?;
    if m.format != "deckcraft" {
        return Err(FormatError::NotOurs(m.format));
    }
    let mut p = m.presentation;
    for item in &mut p.media {
        if let Ok(data) = read_entry(&mut z, &format!("media/{}", item.id.0)) {
            item.data = Arc::new(data);
        }
    }
    repair(&mut p);
    Ok(p)
}

/// Make a loaded presentation safe to edit: a master exists, every slide's layout exists,
/// ids are unique, tables are rectangular.
pub fn repair(p: &mut Presentation) {
    if p.masters.is_empty() {
        let fresh = defaults::blank_presentation(p.slide_size, Default::default(), false);
        p.masters = fresh.masters;
    }
    p.fix_next_id();
    let first_layout = p.masters.first().and_then(|m| m.layouts.first()).map(|l| l.id).unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    for i in 0..p.slides.len() {
        let needs_layout = p.slides.get(i).is_some_and(|s| p.layout(s.layout).is_none());
        let dup = p.slides.get(i).is_some_and(|s| !seen.insert(s.id));
        if needs_layout || dup {
            let new_id = if dup { Some(deckcraft_model::SlideId(p.alloc_id())) } else { None };
            if let Some(s) = p.slides.get_mut(i) {
                let s = Arc::make_mut(s);
                if needs_layout {
                    s.layout = first_layout;
                }
                if let Some(id) = new_id {
                    s.id = id;
                }
            }
        }
    }
    if !(p.slide_size.width.is_finite() && p.slide_size.width >= 1.0 && p.slide_size.height.is_finite() && p.slide_size.height >= 1.0) {
        p.slide_size = defaults::WIDE;
    }
    p.slide_size.width = p.slide_size.width.min(4000.0);
    p.slide_size.height = p.slide_size.height.min(4000.0);
}

/// Slides read from an outline: each title with its body paragraphs (level 0–8, text).
type OutlineSlides = Vec<(String, Vec<(u8, String)>)>;

/// Append a Title and Content slide per outline entry; the number made.
fn add_outline_slides(p: &mut Presentation, slides: OutlineSlides) -> usize {
    let Some(lay) = defaults::layout_of_kind(p, LayoutType::TitleAndContent) else { return 0 };
    let made = slides.len();
    for (title, body) in slides {
        let mut s = defaults::new_slide(p, lay);
        if let Some(t) = s.shapes.first_mut() {
            t.text = Some(TextBody::from_text(&title));
        }
        if let Some(b) = s.shapes.get_mut(1) {
            for (level, line) in body {
                let tb = b.text.get_or_insert_with(TextBody::default);
                let mut para = Paragraph::new(&line);
                para.level = level.min(8);
                tb.paragraphs.push(para);
            }
        }
        p.slides.push(Arc::new(s));
    }
    made
}

/// Outline text: one slide per unindented line; tab-indented lines become body bullets at their
/// level (Insert ▸ Slides from Outline, View ▸ Outline copy).
pub fn outline_to_slides(p: &mut Presentation, text: &str) -> usize {
    let mut slides: OutlineSlides = Vec::new();
    for raw in text.lines() {
        if raw.trim().is_empty() {
            continue;
        }
        let tabs = raw.chars().take_while(|c| *c == '\t').count();
        let spaces = raw.chars().take_while(|c| *c == ' ').count() / 2;
        let level = tabs.max(spaces);
        let line = raw.trim().trim_start_matches(['-', '*', '•']).trim().to_string();
        if level == 0 {
            slides.push((line, Vec::new()));
        } else if let Some((_, body)) = slides.last_mut() {
            body.push(((level - 1).min(8) as u8, line));
        }
    }
    add_outline_slides(p, slides)
}

/// An ATX heading's text (CommonMark: up to three spaces, one to six `#`, then a space or the end;
/// a closing run of `#` is dropped).
fn md_heading(line: &str) -> Option<&str> {
    let t = line.strip_prefix("   ").or_else(|| line.strip_prefix("  ")).or_else(|| line.strip_prefix(' ')).unwrap_or(line);
    let hashes = t.chars().take_while(|c| *c == '#').count();
    let rest = t.get(hashes..)?;
    if !(1..=6).contains(&hashes) || !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    let rest = rest.trim();
    let closed = rest.trim_end_matches('#');
    Some(if closed.is_empty() || closed.ends_with([' ', '\t']) { closed.trim_end() } else { rest })
}

/// A list item's nesting level (two spaces or a tab each) and text: `-`, `*` or `+`, or digits
/// then `.` or `)`, followed by a space.
fn md_list_item(line: &str) -> Option<(u8, &str)> {
    let indent: usize = line.chars().take_while(|c| matches!(c, ' ' | '\t')).map(|c| if c == '\t' { 2 } else { 1 }).sum();
    let t = line.trim_start();
    let rest = match t.strip_prefix(['-', '*', '+']) {
        Some(r) => r,
        None => {
            let digits = t.chars().take_while(char::is_ascii_digit).count();
            if digits == 0 || digits > 9 {
                return None;
            }
            t.get(digits..)?.strip_prefix(['.', ')'])?
        }
    };
    rest.starts_with([' ', '\t']).then(|| ((indent / 2).min(8) as u8, rest.trim()))
}

/// Markdown outline text (`.md`): each heading starts a slide with its text as the title; list
/// items under it become bullets nested by their indentation, other lines body paragraphs. Text
/// before the first heading is skipped, as indented lines before the first title are in the
/// plain format. A file without headings is read as a plain outline ([`outline_to_slides`]).
pub fn markdown_to_slides(p: &mut Presentation, text: &str) -> usize {
    if !text.lines().any(|l| md_heading(l).is_some()) {
        return outline_to_slides(p, text);
    }
    let mut slides: OutlineSlides = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(title) = md_heading(line) {
            slides.push((title.to_string(), Vec::new()));
        } else if let Some((_, body)) = slides.last_mut() {
            let (level, text) = md_list_item(line).unwrap_or((0, line.trim()));
            body.push((level, text.to_string()));
        }
    }
    add_outline_slides(p, slides)
}

/// The deck as outline text (titles unindented, body paragraphs tab-indented by level).
pub fn slides_to_outline(p: &Presentation) -> String {
    let mut out = String::new();
    for s in &p.slides {
        out.push_str(&s.title());
        out.push('\n');
        for sh in &s.shapes {
            if sh.ph_type().is_some_and(|k| k.is_title()) {
                continue;
            }
            if let Some(t) = &sh.text {
                for para in &t.paragraphs {
                    if para.is_empty() {
                        continue;
                    }
                    for _ in 0..=para.level {
                        out.push('\t');
                    }
                    out.push_str(&para.text().replace('\u{b}', " "));
                    out.push('\n');
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_with_media() {
        let mut p = Presentation::default();
        let id = p.add_media("a.png", "image/png", vec![1, 2, 3, 4]);
        let bytes = save(&p).unwrap();
        assert!(sniff(&bytes));
        let q = load(&bytes).unwrap();
        assert_eq!(q.slides.len(), p.slides.len());
        assert_eq!(q.media(id).map(|m| m.data.as_slice()), Some(&[1u8, 2, 3, 4][..]));
        assert_eq!(serde_json::to_value(&q.slides).unwrap(), serde_json::to_value(&p.slides).unwrap());
    }

    #[test]
    fn garbage_is_an_error_not_a_panic() {
        assert!(load(b"").is_err());
        assert!(load(b"PK\x03\x04garbage").is_err());
        assert!(load(&[0u8; 1000]).is_err());
        let mut b = save(&Presentation::default()).unwrap();
        let n = b.len();
        b.truncate(n / 2);
        assert!(load(&b).is_err());
    }

    #[test]
    fn repair_fixes_missing_layout_and_size() {
        let mut p = Presentation::default();
        Arc::make_mut(&mut p.slides[0]).layout = deckcraft_model::LayoutId(99999);
        p.slide_size.width = f64::NAN;
        repair(&mut p);
        assert!(p.validate().is_empty());
        assert_eq!(p.slide_size.width, 960.0);
    }

    #[test]
    fn outline_roundtrip() {
        let mut p = defaults::blank_presentation(defaults::WIDE, Default::default(), false);
        let n = outline_to_slides(&mut p, "Intro\n\tPoint one\n\t\tSub point\nSecond\n\tMore");
        assert_eq!(n, 2);
        let o = slides_to_outline(&p);
        assert!(o.contains("Intro\n\tPoint one\n\t\tSub point\nSecond"), "{o}");
    }

    #[test]
    fn markdown_headings_and_list_items() {
        assert_eq!(md_heading("# Intro"), Some("Intro"));
        assert_eq!(md_heading("   ### Deep ###  "), Some("Deep"));
        assert_eq!(md_heading("## C# and F#"), Some("C# and F#"));
        assert_eq!(md_heading("#"), Some(""));
        assert_eq!(md_heading("## Ünïcödé"), Some("Ünïcödé"));
        for not in ["#5 priority", "####### seven", "    # code", "text", "", "é#"] {
            assert_eq!(md_heading(not), None, "{not:?}");
        }
        assert_eq!(md_list_item("- a"), Some((0, "a")));
        assert_eq!(md_list_item("    * b"), Some((2, "b")));
        assert_eq!(md_list_item("\t+ c"), Some((1, "c")));
        assert_eq!(md_list_item("12) d"), Some((0, "d")));
        for not in ["-x", "1.x", "1", "1234567890. long", "• bullet", "é. x", ""] {
            assert_eq!(md_list_item(not), None, "{not:?}");
        }
    }
}
