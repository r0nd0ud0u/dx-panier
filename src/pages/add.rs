use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::{InputUnit, Purchase, normalize, parse_price_to_cents, parse_quantity};
use crate::pages::widgets::{RecentRow, input_unit_label};
use crate::pages::{DATE_FORMAT, today};
use crate::storage::use_db;

/// Data entry. The landing page, because it is the only thing anyone does in a
/// shop: everything else is read later, at home.
#[component]
pub fn AddPage() -> Element {
    let mut db = use_db();

    let mut product = use_signal(String::new);
    let mut store = use_signal(String::new);
    let mut price = use_signal(String::new);
    let mut quantity = use_signal(|| "1".to_owned());
    let mut unit = use_signal(InputUnit::default);
    let mut date = use_signal(|| today().format(DATE_FORMAT).to_string());
    let mut note = use_signal(String::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let known_products = use_memo(move || db().product_names());
    let known_stores = use_memo(move || db().stores());
    let recent = use_memo(move || {
        db().sorted_recent_first()
            .into_iter()
            .take(8)
            .collect::<Vec<_>>()
    });

    let mut submit = move || {
        let product_name = normalize(&product());
        let store_name = normalize(&store());
        if product_name.is_empty() {
            error.set(Some(t!("error-product")));
            return;
        }
        if store_name.is_empty() {
            error.set(Some(t!("error-store")));
            return;
        }
        let Some(price_cents) = parse_price_to_cents(&price()) else {
            error.set(Some(t!("error-price")));
            return;
        };
        let Some(quantity_value) = parse_quantity(&quantity()) else {
            error.set(Some(t!("error-quantity")));
            return;
        };
        let day = NaiveDate::parse_from_str(&date(), DATE_FORMAT).unwrap_or_else(|_| today());
        let note_text = normalize(&note());
        // 250 g is stored as 0.25 kg — see `InputUnit`.
        let (canonical_unit, canonical_quantity) = unit().to_canonical(quantity_value);

        let purchase = Purchase {
            id: db.read().next_id(),
            product: product_name,
            store: store_name,
            price_cents,
            quantity: canonical_quantity,
            unit: canonical_unit,
            date: day,
            note: (!note_text.is_empty()).then_some(note_text),
        };
        db.write().insert(purchase);

        // Store, date and unit deliberately survive: a receipt is many products
        // from one shop on one day, so the next line is two fields away.
        error.set(None);
        product.set(String::new());
        price.set(String::new());
        quantity.set("1".to_owned());
        note.set(String::new());
    };

    rsx! {
        header { class: "page-header",
            h1 { {t!("add-title")} }
        }

        div { class: "form card",
            label { class: "field",
                span { class: "field-label", {t!("field-product")} }
                input {
                    class: "control",
                    "list": "known-products",
                    value: "{product}",
                    placeholder: t!("field-product-placeholder"),
                    autocapitalize: "sentences",
                    oninput: move |event| product.set(event.value()),
                }
            }
            datalist { id: "known-products",
                for name in known_products() {
                    option { key: "{name}", value: "{name}" }
                }
            }

            label { class: "field",
                span { class: "field-label", {t!("field-store")} }
                input {
                    class: "control",
                    "list": "known-stores",
                    value: "{store}",
                    placeholder: t!("field-store-placeholder"),
                    autocapitalize: "words",
                    oninput: move |event| store.set(event.value()),
                }
            }
            datalist { id: "known-stores",
                for name in known_stores() {
                    option { key: "{name}", value: "{name}" }
                }
            }

            div { class: "field-row",
                label { class: "field",
                    span { class: "field-label", {t!("field-price")} }
                    input {
                        class: "control",
                        // `decimal` rather than `number`: it puts a comma on the
                        // Android keypad, which is how a French price is written,
                        // and never attaches a spinner.
                        inputmode: "decimal",
                        value: "{price}",
                        placeholder: "2,45",
                        oninput: move |event| price.set(event.value()),
                    }
                }
                label { class: "field",
                    span { class: "field-label", {t!("field-quantity")} }
                    input {
                        class: "control",
                        inputmode: "decimal",
                        value: "{quantity}",
                        oninput: move |event| quantity.set(event.value()),
                    }
                }
            }

            div { class: "field-row",
                label { class: "field",
                    span { class: "field-label", {t!("field-unit")} }
                    select {
                        class: "control",
                        value: "{unit().key()}",
                        onchange: move |event| {
                            unit.set(InputUnit::from_key(&event.value()).unwrap_or_default())
                        },
                        for option_unit in InputUnit::ALL {
                            option { key: "{option_unit.key()}", value: "{option_unit.key()}",
                                {input_unit_label(option_unit)}
                            }
                        }
                    }
                }
                label { class: "field",
                    span { class: "field-label", {t!("field-date")} }
                    input {
                        class: "control",
                        r#type: "date",
                        value: "{date}",
                        oninput: move |event| date.set(event.value()),
                    }
                }
            }

            label { class: "field",
                span { class: "field-label", {t!("field-note")} }
                input {
                    class: "control",
                    value: "{note}",
                    placeholder: t!("field-note-placeholder"),
                    oninput: move |event| note.set(event.value()),
                }
            }

            if let Some(message) = error() {
                p { class: "error", role: "alert", "{message}" }
            }

            button { class: "button primary", onclick: move |_| submit(), {t!("action-add")} }
        }

        section { class: "section",
            h2 { {t!("recent-title")} }
            if recent().is_empty() {
                p { class: "muted", {t!("recent-empty")} }
            } else {
                ul { class: "list",
                    for purchase in recent() {
                        RecentRow { key: "{purchase.id}", purchase }
                    }
                }
            }
        }
    }
}
