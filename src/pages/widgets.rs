use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::model::{
    InputUnit, Purchase, Trend, Unit, format_cents, format_quantity, format_unit_price,
};
use crate::pages::DATE_FORMAT;
use crate::storage::use_db;

pub fn input_unit_label(unit: InputUnit) -> String {
    match unit {
        InputUnit::Kg => t!("unit-kg"),
        InputUnit::Gram => t!("unit-g"),
        InputUnit::Liter => t!("unit-l"),
        InputUnit::Milliliter => t!("unit-ml"),
        InputUnit::Piece => t!("unit-piece"),
    }
}

/// Where the latest price sits against the one before it.
#[component]
pub fn TrendBadge(trend: Trend) -> Element {
    let (class, arrow, label) = match trend {
        Trend::Up => ("trend up", "↑", t!("trend-up")),
        Trend::Down => ("trend down", "↓", t!("trend-down")),
        Trend::Flat => ("trend flat", "=", t!("trend-flat")),
        Trend::Unknown => ("trend new", "•", t!("trend-new")),
    };
    rsx! {
        span { class, title: "{label}",
            span { class: "trend-arrow", "{arrow}" }
            span { class: "trend-label", "{label}" }
        }
    }
}

/// A just-entered line, as listed under the form: what, where, how much.
#[component]
pub fn RecentRow(purchase: Purchase) -> Element {
    let mut db = use_db();
    // Bound before the handler so the closure captures a `u64` instead of moving
    // the whole purchase out from under the markup that still reads it.
    let id = purchase.id;
    rsx! {
        li { class: "row",
            div { class: "row-main",
                span { class: "row-title", "{purchase.product}" }
                span { class: "row-sub", "{purchase.store} · {purchase.date.format(DATE_FORMAT)}" }
            }
            div { class: "row-side",
                span { class: "price", {format_cents(purchase.price_cents)} }
                span { class: "row-sub",
                    {format_unit_price(purchase.unit_price(), purchase.unit)}
                }
            }
            button {
                class: "icon-button",
                "aria-label": t!("action-delete"),
                onclick: move |_| db.write().remove(id),
                "×"
            }
        }
    }
}

/// The same line seen from inside a product: the product name is the page title,
/// so the date and store lead instead.
#[component]
pub fn HistoryRow(purchase: Purchase) -> Element {
    let mut db = use_db();
    let id = purchase.id;
    let note = purchase.note.clone().unwrap_or_default();
    rsx! {
        li { class: "row",
            div { class: "row-main",
                span { class: "row-title", "{purchase.date.format(DATE_FORMAT)} · {purchase.store}" }
                span { class: "row-sub",
                    {format_quantity(purchase.quantity, purchase.unit)}
                    // Mass and volume already carry their SI symbol; a count of
                    // items is the one case needing a translated word after it.
                    if purchase.unit == Unit::Piece {
                        " "
                        {t!("unit-piece")}
                    }
                    if !note.is_empty() {
                        " · {note}"
                    }
                }
            }
            div { class: "row-side",
                span { class: "price", {format_cents(purchase.price_cents)} }
                span { class: "row-sub",
                    {format_unit_price(purchase.unit_price(), purchase.unit)}
                }
            }
            button {
                class: "icon-button",
                "aria-label": t!("action-delete"),
                onclick: move |_| db.write().remove(id),
                "×"
            }
        }
    }
}

/// Unit price over time, as a filled line chart.
///
/// Hand-rolled SVG rather than a charting crate: the whole requirement is one
/// series of at most a few dozen points, and an inline `<svg>` needs no extra
/// dependency, styles itself from the same custom properties as everything else,
/// and renders identically in a browser and in a webview.
#[component]
pub fn Sparkline(points: Vec<(NaiveDate, f64)>, unit: Unit) -> Element {
    const WIDTH: f64 = 320.0;
    const HEIGHT: f64 = 120.0;
    const PAD: f64 = 10.0;

    if points.len() < 2 {
        return rsx! {
            p { class: "muted", {t!("detail-single-point")} }
        };
    }

    let values: Vec<f64> = points.iter().map(|(_, value)| *value).collect();
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // A flat series would divide by zero; centring it is the honest rendering of
    // "this price never moved".
    let span = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        max - min
    };

    let first_day = points[0].0;
    let last_day = points[points.len() - 1].0;
    let days = (last_day - first_day).num_days() as f64;

    let coords: Vec<(f64, f64)> = points
        .iter()
        .enumerate()
        .map(|(index, (date, value))| {
            // Spaced by elapsed time, so a six-month gap looks like one — unless
            // every purchase landed on the same day, where even spacing is the
            // only thing that can be drawn.
            let ratio = if days > 0.0 {
                (*date - first_day).num_days() as f64 / days
            } else {
                index as f64 / (points.len() - 1) as f64
            };
            let x = PAD + ratio * (WIDTH - 2.0 * PAD);
            let y = HEIGHT - PAD - ((value - min) / span) * (HEIGHT - 2.0 * PAD);
            (x, y)
        })
        .collect();

    let line = coords
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");
    let area = format!(
        "{:.1},{HEIGHT} {line} {:.1},{HEIGHT}",
        coords[0].0,
        coords[coords.len() - 1].0
    );

    rsx! {
        div { class: "chart",
            svg {
                view_box: "0 0 {WIDTH} {HEIGHT}",
                preserve_aspect_ratio: "none",
                role: "img",
                polygon { class: "chart-area", points: "{area}" }
                polyline { class: "chart-line", points: "{line}" }
                for (index , (x , y)) in coords.iter().enumerate() {
                    circle { key: "{index}", class: "chart-dot", cx: "{x}", cy: "{y}", r: "2.5" }
                }
            }
            div { class: "chart-scale",
                span { {format_unit_price(max, unit)} }
                span { {format_unit_price(min, unit)} }
            }
        }
    }
}
