//! Routes, bundled assets, and the handful of constants the first paint needs.

use dioxus::prelude::*;

use crate::pages::{AddPage, PlanPage, ProductDetailPage, ProductsPage, SettingsPage, Shell};

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
    #[route("/")]
    AddPage {},
    #[route("/produits")]
    ProductsPage {},
    // Numeric on purpose — see `model::product_key`.
    #[route("/produit/:product_id")]
    ProductDetailPage { product_id: u64 },
    #[route("/ou-acheter")]
    PlanPage {},
    #[route("/reglages")]
    SettingsPage {},
}

pub const FAVICON: Asset = asset!("/assets/favicon.ico");
pub const MAIN_CSS: Asset = asset!("/assets/main.css");
/// Bundled Inter subsets — see [`inter_font_face_css`].
pub const PATH_FONTS: Asset = asset!("/assets/fonts");

/// Background of the very first paint, before any stylesheet applies. Kept in sync
/// by hand with `--bg` in assets/main.css; it is a literal here because this is
/// what the webview shows *before* that file is parsed.
pub const BOOT_BG: &str = "#0f1419";
pub const BOOT_TEXT: &str = "#e6edf3";

/// [`BOOT_BG`] as wry wants it — the colour the native window is painted before it
/// has a document at all. A test keeps the two in step.
pub const BOOT_BG_RGBA: (u8, u8, u8, u8) = (0x0f, 0x14, 0x19, 0xff);

/// The UI font stack: Inter first (bundled below), then the system faces it most
/// resembles, so text laid out before the face is ready still looks like the app.
pub const FONT_STACK: &str =
    "'Inter', 'Segoe UI', Roboto, 'Helvetica Neue', Arial, system-ui, sans-serif";

/// `@font-face` rules for the bundled Inter files.
///
/// Built in Rust rather than written into the stylesheet because the asset pipeline
/// content-hashes the folder name (`fonts-dxh…`), so a hand-written `url()` inside a
/// CSS file could not resolve.
///
/// Two subsets: `latin` covers English, `latin-ext` the accented characters French
/// product names need. `swap` rather than `block`, so text is never invisible.
pub fn inter_font_face_css() -> String {
    const LATIN: &str = "U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD";
    const LATIN_EXT: &str = "U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304, U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF";
    [
        ("normal", "latin", LATIN),
        ("normal", "latin-ext", LATIN_EXT),
        ("italic", "latin", LATIN),
        ("italic", "latin-ext", LATIN_EXT),
    ]
    .iter()
    .map(|(style, subset, range)| {
        format!(
            "@font-face{{font-family:'Inter';font-style:{style};font-weight:100 900;\
             font-display:swap;src:url('{PATH_FONTS}/inter-{subset}-{style}.woff2') format('woff2');\
             unicode-range:{range};}}"
        )
    })
    .collect()
}

/// The smallest rule set that makes the window look like the app rather than a
/// blank page, inlined into the native clients' boot `<head>`: main.css arrives a
/// beat later, and the gap used to show as a white flash.
pub fn boot_critical_css() -> String {
    format!(
        "html,body{{margin:0;background:{BOOT_BG};color:{BOOT_TEXT};font-family:{FONT_STACK};}}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_boot_background_matches_its_rgba_form() {
        let (r, g, b, a) = BOOT_BG_RGBA;
        assert_eq!(BOOT_BG, format!("#{r:02x}{g:02x}{b:02x}"));
        assert_eq!(a, 0xff);
    }
}
