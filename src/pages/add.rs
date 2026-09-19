use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::{
    InputUnit, Purchase, format_number, normalize, parse_price_to_cents, parse_quantity,
};
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
    // Only meaningful for InputUnit::Piece — see the field's comment below.
    let mut piece_weight = use_signal(String::new);
    let mut date = use_signal(|| today().format(DATE_FORMAT).to_string());
    let mut note = use_signal(String::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    // Reset right after applying a pack — a select value that changes on its
    // own, from the chosen id back to "", is what makes Dioxus re-render the
    // dropdown back to its placeholder instead of leaving it stuck on the pack
    // just applied.
    let mut chosen_pack = use_signal(String::new);

    let known_products = use_memo(move || db().product_names());
    let known_stores = use_memo(move || db().stores());
    let matching_packs = use_memo(move || {
        let name = product();
        if name.trim().is_empty() {
            Vec::new()
        } else {
            db().packs_for(&name)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        }
    });
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
        // Blank is "not given", not zero — the field is optional and only shown
        // for InputUnit::Piece to begin with.
        let piece_weight_kg = if unit() == InputUnit::Piece && !piece_weight().trim().is_empty() {
            let Some(weight) = parse_quantity(&piece_weight()) else {
                error.set(Some(t!("error-piece-weight")));
                return;
            };
            Some(weight)
        } else {
            None
        };
        let day = NaiveDate::parse_from_str(&date(), DATE_FORMAT).unwrap_or_else(|_| today());
        let note_text = normalize(&note());
        // 250 g is stored as 0.25 kg, and 4 pieces at 150 g each as 0.6 kg — see
        // `InputUnit::to_canonical_weighted`.
        let (canonical_unit, canonical_quantity) =
            unit().to_canonical_weighted(quantity_value, piece_weight_kg);

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
        // Unlike store/date/unit, a per-item weight describes this specific
        // batch (these kiwis, today), not the shopping trip — it does not
        // survive to the next line.
        piece_weight.set(String::new());
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

            // A one-shot shortcut, not an ongoing state: applying a pack fills
            // Quantité/Unité below, then the select snaps back to its
            // placeholder — see `chosen_pack`'s comment above. Packs are
            // defined in Réglages.
            if !matching_packs().is_empty() {
                label { class: "field",
                    span { class: "field-label", {t!("field-pack")} }
                    select {
                        class: "control",
                        value: "{chosen_pack}",
                        onchange: move |event| {
                            let chosen = event.value();
                            if let Some(pack) = matching_packs().into_iter().find(|p| p.id.to_string() == chosen)
                            {
                                quantity.set(format_number(pack.pieces));
                                unit.set(InputUnit::Piece);
                            }
                            chosen_pack.set(String::new());
                        },
                        option { value: "", {t!("field-pack-none")} }
                        for pack in matching_packs() {
                            option { key: "{pack.id}", value: "{pack.id}",
                                "{pack.label} · {format_number(pack.pieces)} "
                                {t!("unit-piece")}
                            }
                        }
                    }
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

            // Lets a per-item purchase (kiwis sold loose, "4 pièces") join the
            // same comparison as one entered by weight ("600 g") — see
            // `InputUnit::to_canonical_weighted`. Shown only for Piece: a
            // weight already means something else — the quantity itself — for
            // every other unit.
            if unit() == InputUnit::Piece {
                label { class: "field",
                    span { class: "field-label", {t!("field-piece-weight")} }
                    input {
                        class: "control",
                        inputmode: "decimal",
                        value: "{piece_weight}",
                        // A literal example, not translated — see the price
                        // field's placeholder just below for the same choice.
                        placeholder: "0,150",
                        oninput: move |event| piece_weight.set(event.value()),
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
