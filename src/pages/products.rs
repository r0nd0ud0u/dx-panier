use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::common::Route;
use crate::model::{fold_key, format_cents, format_unit_price, product_key};
use crate::pages::widgets::TrendBadge;
use crate::storage::{use_db, use_derived};

/// Everything bought so far, one row per product, with the cheapest store named.
#[component]
pub fn ProductsPage() -> Element {
    let db = use_db();
    let mut search = use_signal(String::new);
    let mut store_filter = use_signal(String::new);

    // `db.read()` throughout, never `db()`: a Signal derefs to `Fn() -> T`, so
    // calling it clones the entire database — every purchase, every string —
    // once per call site, per render.
    let derived = use_derived();
    let stores = derived.stores;
    // The unfiltered comparison is already cached by the layout; only a store
    // filter, which most visits do not use, needs its own pass.
    let summaries = use_memo(move || {
        let filter = store_filter();
        if filter.is_empty() {
            derived.summaries.read().clone()
        } else {
            db.read().summaries(Some(&filter))
        }
    });
    let matches = use_memo(move || {
        let needle = fold_key(&search());
        if needle.is_empty() {
            return summaries.read().clone();
        }
        summaries
            .read()
            .iter()
            .filter(|summary| fold_key(&summary.product).contains(&needle))
            .cloned()
            .collect::<Vec<_>>()
    });
    let is_empty = use_memo(move || db.read().purchases.is_empty());
    // Every row rendered is a dozen DOM nodes crossing the bridge to the
    // webview, and a phone shows six at a time. The rest arrive on demand.
    let mut shown = use_signal(|| PAGE);

    rsx! {
        header { class: "page-header",
            h1 { {t!("products-title")} }
        }

        div { class: "filters",
            input {
                class: "control",
                r#type: "search",
                value: "{search}",
                placeholder: t!("products-search"),
                oninput: move |event| search.set(event.value()),
            }
            select {
                class: "control",
                value: "{store_filter}",
                onchange: move |event| store_filter.set(event.value()),
                option { value: "", {t!("products-store-all")} }
                for name in stores() {
                    option { key: "{name}", value: "{name}", "{name}" }
                }
            }
        }

        if is_empty() {
            p { class: "muted", {t!("products-empty")} }
        } else if matches().is_empty() {
            p { class: "muted", {t!("products-no-match")} }
        } else {
            ul { class: "list",
                for summary in matches().into_iter().take(shown()) {
                    li { key: "{summary.product}", class: "card-row",
                        Link {
                            to: Route::ProductDetailPage {
                                product_id: product_key(&summary.product),
                            },
                            class: "card-link",
                            div { class: "row-main",
                                span { class: "row-title", "{summary.product}" }
                                span { class: "row-sub",
                                    {t!("label-purchases", count : summary.purchase_count as i64)}
                                }
                                if let Some(best) = summary.best() {
                                    span { class: "best",
                                        {t!("label-best")}
                                        " · {best.store} · "
                                        {format_unit_price(best.avg_unit_price, summary.unit)}
                                    }
                                }
                            }
                            div { class: "row-side",
                                span { class: "price", {format_cents(summary.latest.price_cents)} }
                                span { class: "row-sub",
                                    {format_unit_price(summary.latest.unit_price(), summary.unit)}
                                }
                                TrendBadge { trend: summary.trend() }
                            }
                        }
                    }
                }
            }
            if matches().len() > shown() {
                button {
                    class: "button",
                    onclick: move |_| shown += PAGE,
                    {t!("action-show-more", count : (matches().len() - shown()) as i64)}
                }
            }
        }
    }
}

/// How many product rows to render before asking.
const PAGE: usize = 40;
