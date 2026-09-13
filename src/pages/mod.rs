mod add;
mod detail;
mod plan;
mod products;
mod settings;
mod shell;
mod widgets;

pub use add::AddPage;
pub use detail::ProductDetailPage;
pub use plan::PlanPage;
pub use products::ProductsPage;
pub use settings::SettingsPage;
pub use shell::Shell;

use chrono::NaiveDate;

/// The date format used everywhere in the UI: ISO, so it is unambiguous in both
/// locales and sorts the way it reads.
pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// Today, in the device's own timezone.
pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}
