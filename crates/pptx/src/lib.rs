//! PPTX import and export: Office Open XML PresentationML / DrawingML, written from the public
//! ECMA-376 standard.
//!
//! [`import`] reads a package leniently (unknown parts are skipped, malformed parts are logged and
//! ignored, every size and depth is bounded) into the DeckCraft model; [`export`] writes a valid
//! package that PowerPoint opens without repair.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

mod cfb;
mod custgeom;
mod maps;
mod opc;
mod png;
pub(crate) mod read;
mod tables;
mod write;
mod xml;

use deckcraft_model::Presentation;

/// Extension URI of the PowerPoint 2010 section list in `presentation.xml`.
pub(crate) const SECTION_EXT_URI: &str = "{521415D9-36F7-43E2-AB2F-B90AF26B5E84}";

/// Extension URI of the modern-comments anchor in a slide (names the threaded comment part).
pub(crate) const COMMENTS_EXT_URI: &str = "{6950BFC3-D8DA-4A85-94F7-54DA5524770B}";

#[derive(Debug, thiserror::Error)]
pub enum PptxError {
    #[error("not a PPTX file: {0}")]
    NotPptx(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("write failed: {0}")]
    Write(String),
    #[error("malformed XML: {0}")]
    Xml(String),
}

/// Does this look like an OOXML package (zip with `[Content_Types].xml`)?
///
/// The part is looked up in the zip's central directory, not in the leading bytes: writers may
/// store it anywhere in the archive (recent PowerPoint builds put it last, megabytes in).
pub fn sniff(bytes: &[u8]) -> bool {
    if !bytes.starts_with(b"PK") {
        return false;
    }
    zip::ZipArchive::new(std::io::Cursor::new(bytes)).map(|z| z.index_for_name("[Content_Types].xml").is_some()).unwrap_or(false)
}

/// Is this a password-protected presentation (an encrypted package inside a compound file)?
pub fn is_encrypted(bytes: &[u8]) -> bool {
    cfb::is_encrypted(bytes)
}

/// Read a `.pptx` / `.potx` / `.ppsx` package.
pub fn import(bytes: &[u8]) -> Result<Presentation, PptxError> {
    read::import(bytes)
}

/// Write a `.pptx` package.
pub fn export(p: &Presentation) -> Result<Vec<u8>, PptxError> {
    write::export(p, write::CT_PRES)
}

/// Write a package for `ext`: `potx` (template) and `ppsx` (slide show) differ from `pptx` only in
/// the main part's content type.
pub fn export_as(p: &Presentation, ext: &str) -> Result<Vec<u8>, PptxError> {
    let ct = match ext {
        "potx" => write::CT_TEMPLATE,
        "ppsx" => write::CT_SHOW,
        _ => write::CT_PRES,
    };
    write::export(p, ct)
}

/// Content type for a file name's extension.
pub(crate) fn mime_for(name: &str) -> &'static str {
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" | "jpe" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        "emf" => "image/x-emf",
        "wmf" => "image/x-wmf",
        "webp" => "image/webp",
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",
        "avi" => "video/x-msvideo",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "wma" => "audio/x-ms-wma",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "fntdata" => "application/x-fontdata",
        _ => "application/octet-stream",
    }
}
