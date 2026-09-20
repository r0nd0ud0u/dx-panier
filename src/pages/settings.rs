use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::{Db, PackDef, format_number, normalize, parse_quantity};
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
    let mut product = use_signal(String::new);
    let mut label = use_signal(String::new);
    let mut pieces = use_signal(String::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let known_products = use_memo(move || db().product_names());
    let packs = use_memo(move || db().packs.clone());

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

#[derive(Clone, Copy, PartialEq)]
enum Notice {
    ImportDone(usize),
    ImportError,
    ExportDone,
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

/// Android's WebView has a real native file picker and share sheet — but
/// neither `rfd` nor dioxus-desktop's own `<input type="file">` bridge (used
/// above on web and desktop) reach them: dioxus intercepts every file input
/// click itself (`window.addEventListener("click", ...)` in its JS runtime,
/// shared by desktop and mobile alike) and asks Rust to run a native dialog —
/// which on Android is a stub that answers with nothing. `import_native`
/// and `export_share` below go around that stub entirely, straight to the
/// browser platform APIs, which is why they need raw JS rather than the
/// dioxus-rendered `<input>` the other `BackupSection` uses.
///
/// Untested on a real device — there was none available to build this
/// against — so the copy-paste flow stays too, revealed only once a button
/// above has actually failed, never hidden behind a guess that it won't.
#[cfg(feature = "mobile")]
#[component]
fn BackupSection(mut db: Signal<Db>, mut notice: Signal<Option<Notice>>) -> Element {
    let mut import_text = use_signal(String::new);
    let export_text = use_memo(move || serde_json::to_string(&db()).unwrap_or_default());

    rsx! {
        section { class: "section",
            h2 { {t!("settings-export")} }
            p { class: "muted", {t!("settings-backup-help")} }
            button {
                class: "button",
                onclick: move |_| export_share(db, notice),
                {t!("action-export")}
            }
            if notice() == Some(Notice::ExportError) {
                p { class: "muted", {t!("settings-export-fallback-help")} }
                textarea { class: "control mono", rows: "4", readonly: true, value: "{export_text}" }
            }
        }

        section { class: "section",
            h2 { {t!("settings-import")} }
            p { class: "muted", {t!("settings-restore-help")} }
            button {
                class: "button",
                onclick: move |_| import_native(db, notice),
                {t!("action-import")}
            }
            if notice() == Some(Notice::ImportError) {
                p { class: "muted", {t!("settings-import-fallback-help")} }
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
        }

        BackupNotice { notice }
    }
}

/// Hands the backup to Android's share sheet ("Save to Drive", "Files", …) as
/// a real file, via the Web Share API's file support (`navigator.share`
/// with `files:`) — a plain script call, not a DOM element dioxus watches,
/// so nothing here needs to dodge its file-input interception.
#[cfg(feature = "mobile")]
fn export_share(db: Signal<Db>, mut notice: Signal<Option<Notice>>) {
    const SCRIPT: &str = r#"
        const json = await dioxus.recv();
        const name = await dioxus.recv();
        const file = new File([json], name, { type: "application/json" });
        if (!navigator.canShare || !navigator.canShare({ files: [file] })) {
            return "unsupported";
        }
        try {
            await navigator.share({ files: [file] });
            return "ok";
        } catch (e) {
            // The user closing the share sheet without picking anything is
            // not a failure worth falling back over.
            return (e && e.name === "AbortError") ? "cancelled" : "error";
        }
    "#;
    let json = serde_json::to_string_pretty(&db()).unwrap_or_default();
    spawn(async move {
        let eval = document::eval(SCRIPT);
        if eval.send(json).is_err() || eval.send(EXPORT_FILE_NAME).is_err() {
            notice.set(Some(Notice::ExportError));
            return;
        }
        let status: String = eval
            .await
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_else(|| "error".to_owned());
        match status.as_str() {
            "ok" => notice.set(Some(Notice::ExportDone)),
            "cancelled" => {}
            _ => notice.set(Some(Notice::ExportError)),
        }
    });
}

/// A JSON-only response from `import_native`'s script: which of "ok",
/// "cancelled" or "error" it ended on, and the file's text when "ok".
#[cfg(feature = "mobile")]
#[derive(serde::Deserialize)]
struct NativeImportResult {
    status: String,
    #[serde(default)]
    text: String,
}

/// Opens Android's own file picker directly — bypassing dioxus's own
/// file-input handling, which never reaches it (see `BackupSection`'s
/// comment) — by creating a plain, undecorated `<input type="file">` outside
/// dioxus's render tree and stopping its click from ever reaching the
/// `window`-level listener dioxus installs. Cached on `window` so a second
/// import reuses the same element rather than leaking one per attempt.
#[cfg(feature = "mobile")]
fn import_native(mut db: Signal<Db>, mut notice: Signal<Option<Notice>>) {
    const SCRIPT: &str = r#"
        return await new Promise((resolve) => {
            if (!window.__panierFileInput) {
                const input = document.createElement("input");
                input.type = "file";
                input.accept = ".json";
                input.style.display = "none";
                input.addEventListener("click", (e) => e.stopPropagation());
                document.body.appendChild(input);
                window.__panierFileInput = input;
            }
            const input = window.__panierFileInput;
            const finish = (result) => {
                input.removeEventListener("change", onChange);
                input.removeEventListener("cancel", onCancel);
                resolve(result);
            };
            const onChange = () => {
                const file = input.files && input.files[0];
                if (!file) { finish({ status: "cancelled" }); return; }
                file.text()
                    .then((text) => finish({ status: "ok", text }))
                    .catch(() => finish({ status: "error" }));
            };
            const onCancel = () => finish({ status: "cancelled" });
            input.addEventListener("change", onChange);
            input.addEventListener("cancel", onCancel);
            // Otherwise picking the same file twice in a row does not fire
            // "change" the second time — its value never appeared to change.
            input.value = "";
            input.click();
        });
    "#;
    spawn(async move {
        let result: Option<NativeImportResult> = document::eval(SCRIPT)
            .await
            .ok()
            .and_then(|value| serde_json::from_value(value).ok());
        match result {
            Some(NativeImportResult { status, text }) if status == "ok" => {
                apply_import(&text, &mut db, &mut notice);
            }
            Some(NativeImportResult { status, .. }) if status == "cancelled" => {}
            _ => notice.set(Some(Notice::ImportError)),
        }
    });
}
