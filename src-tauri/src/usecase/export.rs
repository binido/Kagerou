use qrcode::render::svg;
use qrcode::QrCode;
use thiserror::Error;

use crate::storage::models::Profile;
use crate::storage::{profiles, Db, StorageError};
use crate::subscription;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("the link does not fit in a QR code: {0}")]
    TooLongForQr(qrcode::types::QrError),
}

/// The link a profile is shared as, under the name it has now.
///
/// The stored key keeps the name it was imported with, so a renamed profile
/// would otherwise go out under its old name. A key that no longer parses is
/// shared as stored.
pub fn share_link(profile: &Profile) -> String {
    match subscription::parse_uri(&profile.key) {
        Ok(mut outbound) => {
            outbound.set_name(&profile.name);
            subscription::to_uri(&outbound)
        }
        Err(_) => profile.key.trim().to_string(),
    }
}

/// Share links for the given profiles, in the order given.
pub fn share_links(db: &Db, ids: &[String]) -> Result<Vec<String>, ExportError> {
    ids.iter()
        .map(|id| Ok(share_link(&profiles::get(db, id)?)))
        .collect()
}

/// The profile's share link as an SVG QR code.
///
/// Always dark on light, whatever the theme, because inverted codes are
/// not read by every scanner.
pub fn qr_svg(db: &Db, id: &str) -> Result<String, ExportError> {
    let link = share_link(&profiles::get(db, id)?);
    let code = QrCode::new(link.as_bytes()).map_err(ExportError::TooLongForQr)?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

/// A file name the save dialog can offer: characters no desktop OS allows
/// in a name are replaced.
pub fn suggested_file_name(label: &str) -> String {
    let cleaned: String = label
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let stem = if cleaned.is_empty() {
        "kagerou"
    } else {
        &cleaned
    };
    format!("{stem}.txt")
}

#[cfg(test)]
mod tests;
