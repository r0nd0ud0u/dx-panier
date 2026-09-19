use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::Db;
use crate::storage::{use_db, use_lang};

#[component]
pub fn SettingsPage() -> Element {
    let mut db = use_db();
    let mut lang = use_lang();
    let mut notice: Signal<Option<Notice>> = use_signal(|| None);
    let mut confirm_wipe = use_signal(|| false);

    let stats = use_memo(move || {
        let db = db();
        (
            db.purchases.len(),
            db.product_names().len(),
            db.stores().len(),
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
                {t!("settings-stats", purchases : stats().0.to_string(), products : stats().1
                .to_string(), stores : stats().2.to_string())}
            }
            p { class: "muted", {t!("settings-storage")} }
        }

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

#[derive(Clone, Copy, PartialEq)]
enum Notice {
    ImportDone(usize),
    ImportError,
    // Only web and desktop can report how the export itself went — Android
    // still exports as inert, always-successful text (see `BackupSection`
    // below), so these two never get constructed there.
    #[cfg_attr(feature = "mobile", allow(dead_code))]
    ExportDone,
    #[cfg_attr(feature = "mobile", allow(dead_code))]
    ExportError,
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

#[cfg(not(feature = "mobile"))]
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
    let json = serde_json::to_string_pretty(&db()).unwrap_or_default();
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
    let json = serde_json::to_string_pretty(&db()).unwrap_or_default();
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

/// Android has no file dialog here: `rfd` has no Android backend at all, and
/// dioxus-desktop's own `<input type="file">` bridge (used above on web and
/// desktop) is stubbed out to return nothing on Android too. Until one of
/// those grows Android support, this keeps the copy-paste flow the app has
/// always used there.
#[cfg(feature = "mobile")]
#[component]
fn BackupSection(mut db: Signal<Db>, mut notice: Signal<Option<Notice>>) -> Element {
    let mut import_text = use_signal(String::new);
    let export = use_memo(move || serde_json::to_string(&db()).unwrap_or_default());

    rsx! {
        section { class: "section",
            h2 { {t!("settings-export")} }
            p { class: "muted", {t!("settings-export-help")} }
            textarea { class: "control mono", rows: "4", readonly: true, value: "{export}" }
        }

        section { class: "section",
            h2 { {t!("settings-import")} }
            p { class: "muted", {t!("settings-import-help")} }
            textarea {
                class: "control mono",
                rows: "4",
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

        BackupNotice { notice }
    }
}
