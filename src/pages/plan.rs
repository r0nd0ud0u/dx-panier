use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::common::Route;
use crate::model::{format_unit_price, product_key};
use crate::storage::use_db;

/// The answer to "où j'achète quoi" — every product filed under the store that
/// sells it cheapest, stores ordered by how much the switch is worth.
#[component]
pub fn PlanPage() -> Element {
    let db = use_db();
    let plan = use_memo(move || db().best_store_plan());
    // A product bought at a single store has no comparison in it, so a plan made
    // only of those says nothing — treat it as empty rather than as advice.
    let has_comparison = use_memo(move || {
        plan()
            .iter()
            .any(|(_, products)| products.iter().any(|p| p.stores.len() > 1))
    });

    rsx! {
        header { class: "page-header",
            h1 { {t!("plan-title")} }
        }

        if !has_comparison() {
            p { class: "muted", {t!("plan-empty")} }
        } else {
            p { class: "muted", {t!("plan-intro")} }
            for (store , products) in plan() {
                section { key: "{store}", class: "section",
                    h2 { "{store}" }
                    ul { class: "list",
                        for summary in products {
                            li { key: "{summary.product}", class: "card-row",
                                Link {
                                    to: Route::ProductDetailPage {
                                        product_id: product_key(&summary.product),
                                    },
                                    class: "card-link",
                                    div { class: "row-main",
                                        span { class: "row-title", "{summary.product}" }
                                        if let Some(savings) = summary.savings_per_unit() {
                                            span { class: "best",
                                                {t!("plan-savings", amount : format_unit_price(savings, summary.unit))}
                                            }
                                        }
                                    }
                                    div { class: "row-side",
                                        if let Some(best) = summary.best() {
                                            span { class: "price",
                                                {format_unit_price(best.avg_unit_price, summary.unit)}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
