use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::{Db, PackDef, format_number, normalize, parse_quantity};
use crate::storage::{use_db, use_derived, use_lang};

#[component]
pub fn SettingsPage() -> Element {
    let mut db = use_db();
    let mut lang = use_lang();
    let mut notice: Signal<Option<Notice>> = use_signal(|| None);
    let mut confirm_wipe = use_signal(|| false);

    let derived = use_derived();
    let stats = use_memo(move || {
        (
            db.read().purchases.len(),
            derived.product_names.read().len(),
            derived.stores.read().len(),
        )
    });

    rsx! {
        header { class: "page-header",
            h1 { {t!("settings-title")} }
        }

        section { class: "section",
            h2 { {t!("settings-language")} }
            select {
                class: "control",
                value: "{lang}",
                onchange: move |event| lang.set(event.value()),
                option { value: "fr", "Français" }
                option { value: "en", "English" }
            }
        }

        section { class: "section",
            h2 { {t!("settings-data")} }
            p { class: "muted",
                {
                    t!(
                        "settings-stats", purchases : stats().0.to_string(), products : stats().1
                        .to_string(), stores : stats().2.to_string()
                    )
                }
            }
            p { class: "muted", {t!("settings-storage")} }
        }

        PacksSection { db }

        BackupSection { db, notice }

        section { class: "section",
            h2 { {t!("settings-wipe")} }
            if confirm_wipe() {
                p { class: "error", {t!("settings-wipe-warning")} }
                div { class: "field-row",
                    button {
                        class: "button",
                        onclick: move |_| confirm_wipe.set(false),
                        {t!("action-cancel")}
                    }
                    button {
                        class: "button destructive",
                        onclick: move |_| {
                            db.set(Db::default());
                            confirm_wipe.set(false);
                            notice.set(None);
                        },
                        {t!("action-confirm")}
                    }
                }
            } else {
                button {
                    class: "button destructive",
                    onclick: move |_| confirm_wipe.set(true),
                    {t!("settings-wipe")}
                }
            }
        }
    }
}

/// Named pack sizes ("Pack de 6" for Coca-Cola) that the add form offers as a
/// quantity shortcut for that product — see `matching_packs` in add.rs.
#[component]
fn PacksSection(mut db: Signal<Db>) -> Element {
    let known_products = use_derived().product_names;
    let mut product = use_signal(String::new);
    let mut label = use_signal(String::new);
    let mut pieces = use_signal(String::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let packs = use_memo(move || db.read().packs.clone());

    let mut add_pack = move || {
        let product_name = normalize(&product());
        let label_text = normalize(&label());
        if product_name.is_empty() {
            error.set(Some(t!("error-product")));
            return;
        }
        if label_text.is_empty() {
            error.set(Some(t!("error-pack-label")));
            return;
        }
        let Some(pieces_value) = parse_quantity(&pieces()) else {
            error.set(Some(t!("error-pack-pieces")));
            return;
        };
        let id = db.read().next_pack_id();
        db.write().insert_pack(PackDef {
            id,
            product: product_name,
            label: label_text,
            pieces: pieces_value,
        });
        error.set(None);
        product.set(String::new());
        label.set(String::new());
        pieces.set(String::new());
    };

    rsx! {
        section { class: "section",
            h2 { {t!("settings-packs")} }
            p { class: "muted", {t!("settings-packs-help")} }

            if packs().is_empty() {
                p { class: "muted", {t!("settings-packs-empty")} }
            } else {
                ul { class: "list",
                    for pack in packs() {
                        li { key: "{pack.id}", class: "row",
                            div { class: "row-main",
                                span { class: "row-title", "{pack.product}" }
                                span { class: "row-sub",
                                    "{pack.label} · {format_number(pack.pieces)} "
                                    {t!("unit-piece")}
                                }
                            }
                            button {
                                class: "icon-button",
                                "aria-label": t!("action-delete"),
                                onclick: move |_| db.write().remove_pack(pack.id),
                                "×"
                            }
                        }
                    }
                }
            }

            div { class: "field-row",
                label { class: "field",
                    span { class: "field-label", {t!("field-product")} }
                    input {
                        class: "control",
                        "list": "known-products-for-packs",
                        value: "{product}",
                        placeholder: t!("field-product-placeholder"),
                        oninput: move |event| product.set(event.value()),
                    }
                }
                label { class: "field",
                    span { class: "field-label", {t!("field-pack-label")} }
                    input {
                        class: "control",
                        value: "{label}",
                        placeholder: "Pack de 6",
                        oninput: move |event| label.set(event.value()),
                    }
                }
            }
            datalist { id: "known-products-for-packs",
                for name in known_products() {
                    option { key: "{name}", value: "{name}" }
                }
            }

            label { class: "field",
                span { class: "field-label", {t!("field-pack-pieces")} }
                input {
                    class: "control",
                    inputmode: "decimal",
                    value: "{pieces}",
                    placeholder: "6",
                    oninput: move |event| pieces.set(event.value()),
                }
            }

            if let Some(message) = error() {
                p { class: "error", role: "alert", "{message}" }
            }

            button { class: "button", onclick: move |_| add_pack(), {t!("action-add")} }
        }
    }
}

#[derive(Clone, PartialEq)]
enum Notice {
    ImportDone(usize),
    ImportError,
    /// Web and desktop only: there the file went wherever the dialog said.
    #[cfg_attr(feature = "mobile", allow(dead_code))]
    ExportDone,
    ExportError,
    /// Mobile only: where the file actually landed, which is the one thing the
    /// user needs after an export that had no dialog to pick a location in.
    #[cfg_attr(not(feature = "mobile"), allow(dead_code))]
    ExportSaved(String),
}

#[component]
fn BackupNotice(notice: Signal<Option<Notice>>) -> Element {
    match notice() {
        Some(Notice::ImportDone(count)) => rsx! {
            p { class: "notice", {t!("settings-import-done", count : count as i64)} }
        },
        Some(Notice::ImportError) => rsx! {
            p { class: "error", {t!("settings-import-error")} }
        },
        Some(Notice::ExportDone) => rsx! {
            p { class: "notice", {t!("settings-export-done")} }
        },
        Some(Notice::ExportError) => rsx! {
            p { class: "error", {t!("settings-export-error")} }
        },
        Some(Notice::ExportSaved(path)) => rsx! {
            p { class: "notice", {t!("settings-export-saved")} }
            p { class: "muted mono", "{path}" }
        },
        None => rsx! {},
    }
}

/// Replaces the whole history with what a backup file (or, on mobile, a pasted
/// blob of JSON) contains. Parsed in full before anything is replaced: a
/// truncated file must leave the existing history untouched rather than
/// half-overwrite it.
fn apply_import(text: &str, db: &mut Signal<Db>, notice: &mut Signal<Option<Notice>>) {
    match serde_json::from_str::<Db>(text) {
        Ok(restored) => {
            let count = restored.purchases.len();
            db.set(restored);
            notice.set(Some(Notice::ImportDone(count)));
        }
        Err(_) => notice.set(Some(Notice::ImportError)),
    }
}

/// Export and import as actual files — everywhere a file dialog exists.
///
/// A hidden `<input type="file">` behind a `<label class="button">` reads a
/// picked file through the very same [`dioxus::html::FileData`] whether the
/// picker is the browser's own (web) or dioxus-desktop's native one backed by
/// `rfd` (desktop) — dioxus already abstracts that difference, so this one
/// body serves both. Only the *save* side has no such shared abstraction —
/// `EXPORT_FILE_NAME` and its cfg-gated `export` below fill that gap per
/// platform.
#[cfg(not(feature = "mobile"))]
#[component]
fn BackupSection(mut db: Signal<Db>, mut notice: Signal<Option<Notice>>) -> Element {
    rsx! {
        section { class: "section",
            h2 { {t!("settings-export")} }
            p { class: "muted", {t!("settings-backup-help")} }
            button { class: "button", onclick: move |_| export(db, notice), {t!("action-export")} }
        }

        section { class: "section",
            h2 { {t!("settings-import")} }
            p { class: "muted", {t!("settings-restore-help")} }
            label { class: "button", r#for: "import-file", {t!("action-import")} }
            input {
                id: "import-file",
                class: "sr-only",
                r#type: "file",
                accept: ".json",
                onchange: move |event| {
                    let Some(file) = event.files().into_iter().next() else { return };
                    spawn(async move {
                        match file.read_string().await {
                            Ok(text) => apply_import(&text, &mut db, &mut notice),
                            Err(_) => notice.set(Some(Notice::ImportError)),
                        }
                    });
                },
            }
        }

        BackupNotice { notice }
    }
}

const EXPORT_FILE_NAME: &str = "panier-sauvegarde.json";

/// Downloads the backup the way a browser downloads anything: a `Blob` behind
/// an object URL, clicked through a throwaway `<a download>`. `Eval::send`
/// hands the JSON over as a value on `dioxus.recv()`, not formatted into the
/// script — a product note containing a quote or backslash must not become
/// broken (or malicious) JavaScript.
#[cfg(feature = "web")]
fn export(db: Signal<Db>, mut notice: Signal<Option<Notice>>) {
    const SCRIPT: &str = r#"
        const json = await dioxus.recv();
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = await dioxus.recv();
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
        // Eval::await needs a JSON-serializable result to resolve — an implicit
        // `undefined` return fails that step (silently as far as this script's
        // own side effects go: the download itself still completes), which
        // otherwise reports the download it as failed.
        return true;
    "#;
    let json = serde_json::to_string_pretty(&*db.read()).unwrap_or_default();
    spawn(async move {
        let eval = document::eval(SCRIPT);
        if eval.send(json).is_err() || eval.send(EXPORT_FILE_NAME).is_err() {
            notice.set(Some(Notice::ExportError));
            return;
        }
        match eval.await {
            Ok(_) => notice.set(Some(Notice::ExportDone)),
            Err(_) => notice.set(Some(Notice::ExportError)),
        }
    });
}

/// Writes the backup wherever the native "Save as" dialog is pointed.
///
/// A cancelled dialog raises no notice at all — same as dismissing any other
/// dialog, not a failure worth reporting.
#[cfg(feature = "desktop")]
fn export(db: Signal<Db>, mut notice: Signal<Option<Notice>>) {
    let json = serde_json::to_string_pretty(&*db.read()).unwrap_or_default();
    spawn(async move {
        let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name(EXPORT_FILE_NAME)
            .save_file()
            .await
        else {
            return;
        };
        match file.write(json.as_bytes()).await {
            Ok(()) => notice.set(Some(Notice::ExportDone)),
            Err(_) => notice.set(Some(Notice::ExportError)),
        }
    });
}

/// Android has no reachable file dialog: dioxus intercepts every
/// `<input type="file">` click itself and asks Rust for a native dialog,
/// which is a no-op there, and the Web Share API is not available either.
/// So this browses the filesystem itself with `std::fs` — pick a folder to
/// write into, or a `.json` to read back — with a one-tap default for the
/// common case of not caring where it goes.
#[cfg(feature = "mobile")]
#[component]
fn BackupSection(mut db: Signal<Db>, mut notice: Signal<Option<Notice>>) -> Element {
    let mut import_text = use_signal(String::new);
    let mut browsing: Signal<Option<BrowseMode>> = use_signal(|| None);

    rsx! {
        section { class: "section",
            h2 { {t!("settings-export")} }
            p { class: "muted", {t!("settings-backup-help")} }
            div { class: "field-row",
                button {
                    class: "button",
                    onclick: move |_| export_to_disk(db, notice),
                    {t!("action-export")}
                }
                button {
                    class: "button",
                    onclick: move |_| browsing.set(Some(BrowseMode::Export)),
                    {t!("action-choose-folder")}
                }
            }
        }

        section { class: "section",
            h2 { {t!("settings-import")} }
            p { class: "muted", {t!("settings-restore-help")} }
            button {
                class: "button",
                onclick: move |_| browsing.set(Some(BrowseMode::Import)),
                {t!("action-browse")}
            }

            p { class: "muted", {t!("settings-import-fallback-help")} }
            textarea {
                class: "control mono",
                rows: "3",
                value: "{import_text}",
                oninput: move |event| import_text.set(event.value()),
            }
            button {
                class: "button",
                disabled: import_text().trim().is_empty(),
                onclick: move |_| {
                    apply_import(&import_text(), &mut db, &mut notice);
                    import_text.set(String::new());
                },
                {t!("action-import")}
            }
        }

        if let Some(mode) = browsing() {
            FileBrowser {
                mode,
                db,
                notice,
                browsing,
            }
        }

        BackupNotice { notice }
    }
}

#[cfg(feature = "mobile")]
#[derive(Clone, Copy, PartialEq)]
enum BrowseMode {
    Import,
    Export,
}

/// Shortcuts to the directories worth starting from. Everything outside the
/// app's own two is subject to Android's storage rules, so an unreadable one
/// is listed but reports itself as such rather than being hidden.
#[cfg(feature = "mobile")]
const BROWSE_ROOTS: [&str; 4] = [
    "/storage/emulated/0",
    "/storage/emulated/0/Download",
    "/storage/emulated/0/Android/data/io.github.r0ndoudou.panier/files",
    "/data/data/io.github.r0ndoudou.panier/files/panier",
];

/// A plain directory listing walked with `std::fs` — the whole file dialog,
/// since the platform will not lend us one.
#[cfg(feature = "mobile")]
#[component]
fn FileBrowser(
    mode: BrowseMode,
    mut db: Signal<Db>,
    mut notice: Signal<Option<Notice>>,
    mut browsing: Signal<Option<BrowseMode>>,
) -> Element {
    let mut dir = use_signal(|| std::path::PathBuf::from(default_export_dir()));
    let mut file_name = use_signal(|| EXPORT_FILE_NAME.to_owned());

    // (label, full path) pairs: the closures below move the path out, so the
    // label has to be a separate field to stay readable by the markup.
    let entries = use_memo(move || list_dir(&dir()));

    rsx! {
        section { class: "section",
            h2 { {t!("settings-browse")} }

            div { class: "field-row",
                for root in BROWSE_ROOTS {
                    button {
                        key: "{root}",
                        class: "button",
                        onclick: move |_| dir.set(std::path::PathBuf::from(root)),
                        {short_name(root)}
                    }
                }
            }

            p { class: "muted mono", "{dir().display()}" }

            match entries() {
                Err(message) => rsx! {
                    p { class: "error", "{message}" }
                },
                Ok(listing) => rsx! {
                    ul { class: "list",
                        if let Some(parent) = dir().parent().map(|p| p.to_path_buf()) {
                            li { class: "row",
                                button { class: "button", onclick: move |_| dir.set(parent.clone()), ".." }
                            }
                        }
                        for entry in listing {
                            li { key: "{entry.path}", class: "row",
                                div { class: "row-main",
                                    span { class: "row-title",
                                        if entry.is_dir {
                                            "📁 "
                                        }
                                        "{entry.label}"
                                    }
                                }
                                if entry.is_dir {
                                    button {
                                        class: "button",
                                        onclick: move |_| dir.set(std::path::PathBuf::from(entry.path.clone())),
                                        {t!("action-open")}
                                    }
                                } else if mode == BrowseMode::Import {
                                    button {
                                        class: "button",
                                        onclick: move |_| {
                                            match std::fs::read_to_string(&entry.path) {
                                                Ok(text) => {
                                                    apply_import(&text, &mut db, &mut notice);
                                                    browsing.set(None);
                                                }
                                                Err(_) => notice.set(Some(Notice::ImportError)),
                                            }
                                        },
                                        {t!("action-import")}
                                    }
                                }
                            }
                        }
                    }
                },
            }

            if mode == BrowseMode::Export {
                label { class: "field",
                    span { class: "field-label", {t!("field-file-name")} }
                    input {
                        class: "control",
                        value: "{file_name}",
                        oninput: move |event| file_name.set(event.value()),
                    }
                }
                button {
                    class: "button primary",
                    onclick: move |_| {
                        if write_backup(&dir().join(file_name()), db, notice) {
                            browsing.set(None);
                        }
                    },
                    {t!("action-save-here")}
                }
            }

            button { class: "button", onclick: move |_| browsing.set(None), {t!("action-cancel")} }
        }
    }
}

#[cfg(feature = "mobile")]
#[derive(Clone, PartialEq)]
struct Entry {
    label: String,
    path: String,
    is_dir: bool,
}

/// Directories first, then `.json` files — nothing else is useful here, and a
/// phone's storage root is otherwise a wall of noise.
#[cfg(feature = "mobile")]
fn list_dir(dir: &std::path::Path) -> Result<Vec<Entry>, String> {
    let read = std::fs::read_dir(dir).map_err(|error| error.to_string())?;
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        let Some(label) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        let item = Entry {
            label,
            path: path.to_string_lossy().into_owned(),
            is_dir: path.is_dir(),
        };
        if item.is_dir {
            dirs.push(item);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            files.push(item);
        }
    }
    dirs.sort_by(|a, b| a.label.cmp(&b.label));
    files.sort_by(|a, b| a.label.cmp(&b.label));
    dirs.extend(files);
    Ok(dirs)
}

#[cfg(feature = "mobile")]
fn short_name(path: &str) -> String {
    match path {
        "/storage/emulated/0" => "Stockage".to_owned(),
        _ => std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_owned()),
    }
}

/// Where Android lets this app write without asking for a permission, most
/// useful first. Downloads only works on older releases — scoped storage
/// blocks it from API 29 — and the last one is the app's own private
/// directory, invisible to a file manager but always writable.
#[cfg(feature = "mobile")]
const BACKUP_DIRS: [&str; 3] = [
    "/storage/emulated/0/Download",
    "/storage/emulated/0/Android/data/io.github.r0ndoudou.panier/files",
    "/data/data/io.github.r0ndoudou.panier/files/panier",
];

/// The first of [`BACKUP_DIRS`] that actually accepts a file, so the browser
/// and the one-tap export both open somewhere that works.
#[cfg(feature = "mobile")]
fn default_export_dir() -> &'static str {
    BACKUP_DIRS
        .into_iter()
        .find(|dir| {
            std::fs::create_dir_all(dir).is_ok()
                && std::fs::write(std::path::Path::new(dir).join(".panier-probe"), b"").is_ok()
        })
        .inspect(|dir| {
            let _ = std::fs::remove_file(std::path::Path::new(dir).join(".panier-probe"));
        })
        .unwrap_or(BACKUP_DIRS[BACKUP_DIRS.len() - 1])
}

/// Reports the full path on success — without it the file would be saved
/// somewhere the user has no way to guess.
#[cfg(feature = "mobile")]
fn write_backup(
    path: &std::path::Path,
    db: Signal<Db>,
    mut notice: Signal<Option<Notice>>,
) -> bool {
    let json = serde_json::to_string_pretty(&*db.read()).unwrap_or_default();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(path, &json) {
        Ok(()) => {
            notice.set(Some(Notice::ExportSaved(
                path.to_string_lossy().into_owned(),
            )));
            true
        }
        Err(_) => {
            notice.set(Some(Notice::ExportError));
            false
        }
    }
}

#[cfg(feature = "mobile")]
fn export_to_disk(db: Signal<Db>, notice: Signal<Option<Notice>>) {
    let path = std::path::Path::new(default_export_dir()).join(EXPORT_FILE_NAME);
    write_backup(&path, db, notice);
}
