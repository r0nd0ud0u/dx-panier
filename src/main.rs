use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use dx_panier::common::{FAVICON, MAIN_CSS, Route, inter_font_face_css};
use dx_panier::storage::use_provide_storage;
use unic_langid::langid;

/// The `<head>` both native clients boot from.
///
/// Everything needed for a correct *first* paint has to be in the initial HTML:
/// `App`'s `document::Link` stylesheets are injected by an effect that runs after
/// that paint, which is what makes an unstyled launch flash.
#[cfg(any(feature = "desktop", feature = "mobile"))]
fn native_boot_head() -> String {
    use dx_panier::common::boot_critical_css;

    let mut head = format!(r#"<link rel="icon" href="{FAVICON}">"#);
    // Inline, first, and request-free: whatever else is still in flight, the
    // window is already the app's own ground in the right font.
    head.push_str(&format!(
        "<style>{}{}</style>",
        boot_critical_css(),
        inter_font_face_css()
    ));
    // A native client streams its first DOM only after `window.onload`, so
    // preloading the upright face here means text is laid out in Inter the first
    // time it is painted, with no swap. `crossorigin` because font fetches are
    // CORS-mode even same-origin; the asset protocol answers with `*`.
    for subset in ["latin", "latin-ext"] {
        head.push_str(&format!(
            r#"<link rel="preload" as="font" type="font/woff2" crossorigin href="{}/inter-{subset}-normal.woff2">"#,
            dx_panier::common::PATH_FONTS
        ));
    }
    head.push_str(&format!(r#"<link rel="stylesheet" href="{MAIN_CSS}">"#));
    head
}

fn main() {
    // Native storage needs its directory before the first hook runs.
    #[cfg(not(target_arch = "wasm32"))]
    dx_panier::storage::init_native_dir();

    #[cfg(feature = "desktop")]
    {
        // `with_icon` below never reaches Wayland — GTK3 implements no per-window
        // icon protocol there. The compositor matches the xdg-shell app_id against
        // an installed .desktop file instead, and GTK3 takes that app_id from
        // `g_get_prgname()`, i.e. basename(argv[0]). `dx serve` runs a hashed copy
        // that matches no .desktop file, so pin the name. Must precede gtk_init().
        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        glib::set_prgname(Some("panier"));

        // `dx serve` opens the window straight through tao/wry, bypassing
        // Dioxus.toml's [bundle].icon (read only by `dx bundle`), so set it here.
        let icon = image::load_from_memory(include_bytes!("../assets/icon-512.png"))
            .expect("assets/icon-512.png must be a valid image")
            .into_rgba8();
        let (width, height) = icon.dimensions();
        let window_icon =
            dioxus_desktop::tao::window::Icon::from_rgba(icon.into_raw(), width, height)
                .expect("assets/icon-512.png must be a valid RGBA icon");

        dioxus::LaunchBuilder::new()
            .with_cfg(
                dioxus_desktop::Config::new()
                    .with_custom_head(native_boot_head())
                    // Painted by the webview before it has a document at all —
                    // without it the window opens white for as long as the first
                    // paint takes.
                    .with_background_color(dx_panier::common::BOOT_BG_RGBA)
                    .with_icon(window_icon),
            )
            .launch(App);
    }

    // Android/iOS. `dioxus::mobile` *is* dioxus-desktop (same webview stack), so
    // the launch gets the same treatment minus the window icon and prgname, which
    // are desktop window-manager concerns.
    #[cfg(all(feature = "mobile", not(feature = "desktop")))]
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::mobile::Config::new()
                .with_custom_head(native_boot_head())
                .with_background_color(dx_panier::common::BOOT_BG_RGBA),
        )
        .launch(App);

    // Web: dx generates index.html and renders `App`'s `document::Link`s into it,
    // so the markup arrives styled with no head to patch.
    #[cfg(not(any(feature = "desktop", feature = "mobile")))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Storage first: it hands back the persisted language that i18n starts from,
    // so the app never paints one locale and swaps to the other.
    let lang = use_provide_storage();

    use_init_i18n(|| {
        I18nConfig::new(langid!("fr-FR"))
            .with_locale((langid!("fr-FR"), include_str!("./i18n/fr-FR.ftl")))
            .with_locale((langid!("en-US"), include_str!("./i18n/en-US.ftl")))
            .with_fallback(langid!("fr-FR"))
    });

    use_effect(move || {
        let mut i18n = i18n();
        i18n.set_language(if lang() == "en" {
            langid!("en-US")
        } else {
            langid!("fr-FR")
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // Bundled Inter. Declared here for web and mobile; the native clients also
        // inline the same rules into their boot head, early enough to matter.
        document::Style { {inter_font_face_css()} }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}
