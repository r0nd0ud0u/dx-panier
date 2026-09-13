use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::common::Route;

/// Frame around every page: the scrolling content, and a bottom tab bar placed
/// within thumb reach — this app is used standing in a supermarket aisle.
#[component]
pub fn Shell() -> Element {
    rsx! {
        div { class: "app",
            main { class: "content", Outlet::<Route> {} }
            nav { class: "tabbar",
                Tab { to: Route::AddPage {}, label: t!("nav-add"), icon: TabIcon::Add }
                Tab { to: Route::ProductsPage {}, label: t!("nav-products"), icon: TabIcon::List }
                Tab { to: Route::PlanPage {}, label: t!("nav-plan"), icon: TabIcon::Basket }
                Tab { to: Route::SettingsPage {}, label: t!("nav-settings"), icon: TabIcon::Sliders }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TabIcon {
    Add,
    List,
    Basket,
    Sliders,
}

#[component]
fn Tab(to: Route, label: String, icon: TabIcon) -> Element {
    rsx! {
        Link { to, class: "tab", active_class: "tab-active",
            Icon { icon }
            span { class: "tab-label", "{label}" }
        }
    }
}

/// Inline SVG rather than an icon font or emoji: emoji render differently on every
/// Android skin, and a webfont would be one more blocking request at launch.
#[component]
fn Icon(icon: TabIcon) -> Element {
    rsx! {
        svg {
            class: "icon",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            match icon {
                TabIcon::Add => rsx! {
                    circle { cx: "12", cy: "12", r: "9" }
                    path { d: "M12 8v8M8 12h8" }
                },
                TabIcon::List => rsx! {
                    path { d: "M9 6h11M9 12h11M9 18h11M4.5 6h.01M4.5 12h.01M4.5 18h.01" }
                },
                TabIcon::Basket => rsx! {
                    path { d: "M4 9h16l-1.7 10.2a1 1 0 0 1-1 .8H6.7a1 1 0 0 1-1-.8z" }
                    path { d: "M8.5 9a3.5 3.5 0 0 1 7 0" }
                },
                TabIcon::Sliders => rsx! {
                    path { d: "M6 4v5M6 13v7M12 4v9M12 17v3M18 4v3M18 11v9" }
                    circle { cx: "6", cy: "11", r: "2" }
                    circle { cx: "12", cy: "15", r: "2" }
                    circle { cx: "18", cy: "9", r: "2" }
                },
            }
        }
    }
}
