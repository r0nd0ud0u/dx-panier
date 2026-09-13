use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::Db;
use crate::storage::{use_db, use_lang};

#[component]
pub fn SettingsPage() -> Element {
    let mut db = use_db();
    let mut lang = use_lang();
    let mut import_text = use_signal(String::new);
    let mut notice: Signal<Option<Notice>> = use_signal(|| None);
    let mut confirm_wipe = use_signal(|| false);

    let export = use_memo(move || serde_json::to_string(&db()).unwrap_or_default());
    let stats = use_memo(move || {
        let db = db();
        (
            db.purchases.len(),
            db.product_names().len(),
            db.stores().len(),
        )
    });

    let mut import = move || {
        // Parsed in full before anything is replaced: a truncated paste must leave
        // the existing history untouched rather than half-overwrite it.
        match serde_json::from_str::<Db>(&import_text()) {
            Ok(restored) => {
                let count = restored.purchases.len();
                db.set(restored);
                import_text.set(String::new());
                notice.set(Some(Notice::Done(count)));
            }
            Err(_) => notice.set(Some(Notice::Error)),
        }
    };

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
                onclick: move |_| import(),
                {t!("action-import")}
            }
            match notice() {
                Some(Notice::Done(count)) => rsx! {
                    p { class: "notice", {t!("settings-import-done", count : count as i64)} }
                },
                Some(Notice::Error) => rsx! {
                    p { class: "error", {t!("settings-import-error")} }
                },
                None => rsx! {},
            }
        }

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
    Done(usize),
    Error,
}
