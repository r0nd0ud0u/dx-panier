use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::format_unit_price;
use crate::pages::DATE_FORMAT;
use crate::pages::widgets::{HistoryRow, Sparkline, TrendBadge};
use crate::storage::use_db;

/// One product: how its unit price moved, how the stores compare, and every line
/// that produced those two answers.
#[component]
pub fn ProductDetailPage(product_id: u64) -> Element {
    let db = use_db();
    // Every hook runs before the early return below — a `return` between two of
    // them would shift the hook order on the next render and panic.
    let summary = use_memo(move || {
        let name = db().product_by_key(product_id)?;
        db().summaries(None)
            .into_iter()
            .find(|summary| summary.product == name)
    });
    let history = use_memo(move || match db().product_by_key(product_id) {
        Some(name) => db().history(&name),
        None => Vec::new(),
    });

    let Some(summary) = summary() else {
        return rsx! {
            p { class: "muted", {t!("detail-not-found")} }
        };
    };

    // Oldest first: a chart reads left to right, while the history list below
    // reads newest first.
    let mut series: Vec<_> = history()
        .iter()
        .filter(|purchase| purchase.unit == summary.unit)
        .map(|purchase| (purchase.date, purchase.unit_price()))
        .collect();
    series.reverse();
    let has_other_units = history()
        .iter()
        .any(|purchase| purchase.unit != summary.unit);

    rsx! {
        header { class: "page-header",
            h1 { "{summary.product}" }
            TrendBadge { trend: summary.trend() }
        }

        section { class: "section",
            h2 { {t!("detail-evolution")} }
            Sparkline { points: series, unit: summary.unit }
        }

        section { class: "section",
            h2 { {t!("detail-by-store")} }
            ul { class: "list",
                for (rank , store) in summary.stores.iter().enumerate() {
                    li {
                        key: "{store.store}",
                        class: if rank == 0 { "row best-row" } else { "row" },
                        div { class: "row-main",
                            span { class: "row-title", "{store.store}" }
                            span { class: "row-sub",
                                {t!("label-purchases", count : store.count as i64)}
                                " · "
                                {t!("label-since", date : store.last_date.format(DATE_FORMAT).to_string())}
                            }
                        }
                        div { class: "row-side",
                            span { class: "price",
                                {format_unit_price(store.avg_unit_price, summary.unit)}
                            }
                            if rank == 0 {
                                span { class: "row-sub", {t!("label-best")} }
                            }
                        }
                    }
                }
            }
            if let Some(savings) = summary.savings_per_unit() {
                p { class: "muted",
                    {t!("plan-savings", amount : format_unit_price(savings, summary.unit))}
                }
            }
        }

        section { class: "section",
            h2 { {t!("detail-history")} }
            if has_other_units {
                p { class: "muted", {t!("detail-mixed-units")} }
            }
            ul { class: "list",
                for purchase in history() {
                    HistoryRow { key: "{purchase.id}", purchase }
                }
            }
        }
    }
}
