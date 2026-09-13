//! The whole domain: one flat list of purchases, and the comparisons derived from it.
//!
//! There is deliberately no separate product or store table. A product exists because
//! it was bought at least once, which keeps data entry to a single form and makes a
//! typo fixable by deleting one line instead of reconciling two tables.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The unit a quantity is expressed in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Kg,
    Liter,
    #[default]
    Piece,
}

impl Unit {
    /// Suffix printed after a unit price, as in the `/kg` of `2,45 €/kg`.
    pub fn suffix(self) -> &'static str {
        match self {
            Unit::Kg => "/kg",
            Unit::Liter => "/L",
            Unit::Piece => "/u",
        }
    }
}

/// What the form offers. Grams and millilitres are entry conveniences, not storage
/// units: a packet says "250 g", but only a price per kilo compares against the
/// 500 g tub at the other shop — so they are converted on the way in and a stored
/// purchase is always in a canonical [`Unit`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputUnit {
    Kg,
    Gram,
    Liter,
    Milliliter,
    #[default]
    Piece,
}

impl InputUnit {
    pub const ALL: [InputUnit; 5] = [
        InputUnit::Kg,
        InputUnit::Gram,
        InputUnit::Liter,
        InputUnit::Milliliter,
        InputUnit::Piece,
    ];

    pub fn key(self) -> &'static str {
        match self {
            InputUnit::Kg => "kg",
            InputUnit::Gram => "g",
            InputUnit::Liter => "l",
            InputUnit::Milliliter => "ml",
            InputUnit::Piece => "piece",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|unit| unit.key() == key)
    }

    /// Converts an entered quantity into the unit it is stored and compared in.
    pub fn to_canonical(self, quantity: f64) -> (Unit, f64) {
        match self {
            InputUnit::Kg => (Unit::Kg, quantity),
            InputUnit::Gram => (Unit::Kg, quantity / 1000.0),
            InputUnit::Liter => (Unit::Liter, quantity),
            InputUnit::Milliliter => (Unit::Liter, quantity / 1000.0),
            InputUnit::Piece => (Unit::Piece, quantity),
        }
    }
}

/// One line on one receipt: this product, at this store, on this day, for this price.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Purchase {
    pub id: u64,
    pub product: String,
    pub store: String,
    /// Integer cents. Money is never an `f64`: 0.1 + 0.2 is not 0.3 and a drifting
    /// total is worse than useless in an app whose whole point is comparing prices.
    pub price_cents: i64,
    pub quantity: f64,
    pub unit: Unit,
    pub date: NaiveDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Purchase {
    /// Price of a single unit, in cents — the only number worth comparing across
    /// stores. Six eggs for 2,40 € beat four for 1,80 € (40 vs 45 cents apiece)
    /// even though the second receipt shows the smaller amount.
    pub fn unit_price(&self) -> f64 {
        if self.quantity > 0.0 {
            self.price_cents as f64 / self.quantity
        } else {
            self.price_cents as f64
        }
    }
}

/// Everything the app stores, serialized as one JSON blob (see [`crate::storage`]).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Db {
    pub purchases: Vec<Purchase>,
}

/// What one store charges for one product, averaged over every visit.
#[derive(Clone, Debug, PartialEq)]
pub struct StorePrice {
    pub store: String,
    /// Mean unit price across every recorded purchase here — one lucky promotion
    /// shouldn't crown a store that is otherwise expensive.
    pub avg_unit_price: f64,
    pub last_date: NaiveDate,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
    Flat,
    /// Bought only once so far — nothing to compare against yet.
    Unknown,
}

/// One product, as shown in a list: what it last cost, and where it is cheapest.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductSummary {
    pub product: String,
    /// Unit of the most recent purchase. Only purchases sharing it are aggregated:
    /// a price per kilo and a price per item are not the same kind of number, and
    /// averaging them would produce a confident, meaningless answer.
    pub unit: Unit,
    pub latest: Purchase,
    pub previous_unit_price: Option<f64>,
    /// Every store selling it, cheapest first. Computed over the whole history even
    /// when the list itself is filtered to one store, so the comparison stays honest.
    pub stores: Vec<StorePrice>,
    pub purchase_count: usize,
}

impl ProductSummary {
    pub fn best(&self) -> Option<&StorePrice> {
        self.stores.first()
    }

    /// What switching from the runner-up to the cheapest store saves, per unit.
    /// `None` while only one store has ever sold this product.
    pub fn savings_per_unit(&self) -> Option<f64> {
        let best = self.stores.first()?;
        let runner_up = self.stores.get(1)?;
        Some(runner_up.avg_unit_price - best.avg_unit_price)
    }

    /// Where the last price sits against the one before it. The 1% dead band keeps
    /// a one-cent rounding difference from being reported as a price rise.
    pub fn trend(&self) -> Trend {
        let Some(previous) = self.previous_unit_price else {
            return Trend::Unknown;
        };
        let latest = self.latest.unit_price();
        if previous <= 0.0 {
            return Trend::Unknown;
        }
        let ratio = (latest - previous) / previous;
        if ratio > 0.01 {
            Trend::Up
        } else if ratio < -0.01 {
            Trend::Down
        } else {
            Trend::Flat
        }
    }
}

/// Collapses runs of whitespace so `"  Lait   demi "` and `"Lait demi"` are one product.
pub fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Grouping key: normalized and case-folded, so `Lidl` and `lidl` are one store.
/// The spelling shown to the user is always the most recent one entered.
pub fn fold_key(value: &str) -> String {
    normalize(value).to_lowercase()
}

impl Db {
    pub fn next_id(&self) -> u64 {
        self.purchases.iter().map(|p| p.id).max().unwrap_or(0) + 1
    }

    pub fn insert(&mut self, purchase: Purchase) {
        self.purchases.push(purchase);
    }

    pub fn remove(&mut self, id: u64) {
        self.purchases.retain(|purchase| purchase.id != id);
    }

    /// Distinct store names, most recently used first — that ordering puts the store
    /// you are standing in at the top of the form's suggestion list.
    pub fn stores(&self) -> Vec<String> {
        self.distinct_by(|purchase| &purchase.store)
    }

    pub fn product_names(&self) -> Vec<String> {
        self.distinct_by(|purchase| &purchase.product)
    }

    fn distinct_by(&self, field: impl Fn(&Purchase) -> &String) -> Vec<String> {
        let mut seen = Vec::new();
        for purchase in self.sorted_recent_first() {
            let value = field(&purchase);
            if !seen
                .iter()
                .any(|kept: &String| fold_key(kept) == fold_key(value))
            {
                seen.push(value.clone());
            }
        }
        seen
    }

    /// Every purchase, newest first. Ties broken by id so the order is total and a
    /// same-day correction still lands after the line it corrects.
    pub fn sorted_recent_first(&self) -> Vec<Purchase> {
        let mut purchases = self.purchases.clone();
        purchases.sort_by(|a, b| b.date.cmp(&a.date).then(b.id.cmp(&a.id)));
        purchases
    }

    /// Full history of one product, newest first, every unit included.
    pub fn history(&self, product: &str) -> Vec<Purchase> {
        let key = fold_key(product);
        self.sorted_recent_first()
            .into_iter()
            .filter(|purchase| fold_key(&purchase.product) == key)
            .collect()
    }

    /// One summary per product, product name ascending.
    ///
    /// `store_filter` narrows *which products appear and which purchase counts as the
    /// latest one*, not the price comparison: filtering to Lidl answers "what have I
    /// bought at Lidl and for how much", while still showing whether Aldi is cheaper.
    pub fn summaries(&self, store_filter: Option<&str>) -> Vec<ProductSummary> {
        let store_filter = store_filter.map(fold_key);
        let mut summaries: Vec<ProductSummary> = self
            .product_names()
            .into_iter()
            .filter_map(|product| self.summarize(&product, store_filter.as_deref()))
            .collect();
        summaries.sort_by_key(|summary| fold_key(&summary.product));
        summaries
    }

    fn summarize(&self, product: &str, store_filter: Option<&str>) -> Option<ProductSummary> {
        let history = self.history(product);
        let visible: Vec<&Purchase> = history
            .iter()
            .filter(|purchase| store_filter.is_none_or(|store| fold_key(&purchase.store) == store))
            .collect();
        let latest = (*visible.first()?).clone();

        // Same unit only — see the `unit` field's comment on ProductSummary.
        let comparable: Vec<&Purchase> = history
            .iter()
            .filter(|purchase| purchase.unit == latest.unit)
            .collect();
        let previous_unit_price = visible
            .iter()
            .skip(1)
            .find(|purchase| purchase.unit == latest.unit)
            .map(|purchase| purchase.unit_price());

        let mut stores: Vec<StorePrice> = Vec::new();
        for purchase in &comparable {
            let key = fold_key(&purchase.store);
            match stores
                .iter_mut()
                .find(|entry| fold_key(&entry.store) == key)
            {
                // `comparable` is newest first, so the first sighting carries the
                // store's latest price and spelling; later ones only feed the mean.
                Some(entry) => {
                    entry.avg_unit_price = (entry.avg_unit_price * entry.count as f64
                        + purchase.unit_price())
                        / (entry.count + 1) as f64;
                    entry.count += 1;
                }
                None => stores.push(StorePrice {
                    store: purchase.store.clone(),
                    avg_unit_price: purchase.unit_price(),
                    last_date: purchase.date,
                    count: 1,
                }),
            }
        }
        stores.sort_by(|a, b| {
            a.avg_unit_price
                .partial_cmp(&b.avg_unit_price)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| fold_key(&a.store).cmp(&fold_key(&b.store)))
        });

        Some(ProductSummary {
            product: latest.product.clone(),
            unit: latest.unit,
            latest,
            previous_unit_price,
            stores,
            purchase_count: history.len(),
        })
    }

    /// Resolves a [`product_key`] back to the product's current display name.
    pub fn product_by_key(&self, key: u64) -> Option<String> {
        self.product_names()
            .into_iter()
            .find(|product| product_key(product) == key)
    }

    /// The shopping plan: every product grouped under the store that sells it
    /// cheapest, stores ordered by how much they save overall.
    pub fn best_store_plan(&self) -> Vec<(String, Vec<ProductSummary>)> {
        let mut plan: Vec<(String, Vec<ProductSummary>)> = Vec::new();
        for summary in self.summaries(None) {
            let Some(best) = summary.best() else { continue };
            let store = best.store.clone();
            match plan
                .iter_mut()
                .find(|(name, _)| fold_key(name) == fold_key(&store))
            {
                Some((_, products)) => products.push(summary),
                None => plan.push((store, vec![summary])),
            }
        }
        plan.sort_by(|a, b| {
            let savings = |products: &Vec<ProductSummary>| {
                products
                    .iter()
                    .filter_map(ProductSummary::savings_per_unit)
                    .sum::<f64>()
            };
            savings(&b.1)
                .partial_cmp(&savings(&a.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        plan
    }
}

/// Stable, URL-safe identity for a product name (FNV-1a over its folded key).
///
/// The detail route needs to name a product, and dioxus-router does not
/// percent-encode plain `:param` segments — a product called `Lait 1/2 écrémé`
/// would produce a URL that no longer round-trips. A number always does, and
/// unlike a purchase id it survives that purchase being deleted.
pub fn product_key(product: &str) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    fold_key(product).bytes().fold(OFFSET, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(PRIME)
    })
}

/// Parses what a human types into a price field: `2,45`, `2.45`, `2` or `2,45 €`.
/// Returns `None` for anything that isn't a non-negative amount, so the form can
/// refuse it rather than silently store a zero.
pub fn parse_price_to_cents(input: &str) -> Option<i64> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '€')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    let amount: f64 = cleaned.parse().ok()?;
    if !amount.is_finite() || amount < 0.0 {
        return None;
    }
    Some((amount * 100.0).round() as i64)
}

/// Same leniency as [`parse_price_to_cents`], for the quantity field. Zero is
/// rejected: it would make every unit price infinite.
pub fn parse_quantity(input: &str) -> Option<f64> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    let quantity: f64 = cleaned.parse().ok()?;
    (quantity.is_finite() && quantity > 0.0).then_some(quantity)
}

/// `1234` -> `12,34 €`. The comma and the trailing symbol are French convention and
/// stay that way in the English locale too: the currency is the euro either way, and
/// a number that changes shape with the UI language is harder to compare, not easier.
pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let cents = cents.abs();
    format!("{sign}{},{:02} €", cents / 100, cents % 100)
}

/// `245.0, Unit::Kg` -> `2,45 €/kg`. Amounts under ten cents keep a third decimal:
/// between two spice jars, 0,004 and 0,009 €/g is the whole comparison.
pub fn format_unit_price(cents_per_unit: f64, unit: Unit) -> String {
    if cents_per_unit.abs() < 10.0 {
        let amount = format!("{:.3}", cents_per_unit / 100.0).replace('.', ",");
        format!("{amount} €{}", unit.suffix())
    } else {
        format!(
            "{}{}",
            format_cents(cents_per_unit.round() as i64),
            unit.suffix()
        )
    }
}

/// A plain number, comma-separated and without trailing zeros: `1,5`, `250`, `0,75`.
pub fn format_number(value: f64) -> String {
    let rounded = format!("{value:.3}");
    let trimmed = rounded.trim_end_matches('0').trim_end_matches('.');
    trimmed.replace('.', ",")
}

/// Renders a stored quantity the way it was most likely read off the packet: a
/// quarter kilo is `250 g`, not `0,25 kg`.
///
/// Mass and volume come back with their SI symbol, which is the same in every
/// locale. A count of items does not — it returns the bare number, and the caller
/// appends the translated unit name.
pub fn format_quantity(quantity: f64, unit: Unit) -> String {
    match unit {
        Unit::Kg if quantity < 1.0 => format!("{} g", format_number(quantity * 1000.0)),
        Unit::Kg => format!("{} kg", format_number(quantity)),
        Unit::Liter if quantity < 1.0 => format!("{} ml", format_number(quantity * 1000.0)),
        Unit::Liter => format!("{} L", format_number(quantity)),
        Unit::Piece => format_number(quantity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, day).unwrap()
    }

    fn purchase(id: u64, product: &str, store: &str, cents: i64, qty: f64, day: u32) -> Purchase {
        Purchase {
            id,
            product: product.to_owned(),
            store: store.to_owned(),
            price_cents: cents,
            quantity: qty,
            unit: Unit::Kg,
            date: date(day),
            note: None,
        }
    }

    #[test]
    fn unit_price_makes_different_pack_sizes_comparable() {
        let six_eggs = purchase(1, "Oeufs", "Lidl", 240, 6.0, 1);
        let four_eggs = purchase(2, "Oeufs", "Aldi", 180, 4.0, 2);
        assert_eq!(six_eggs.unit_price(), 40.0);
        assert_eq!(four_eggs.unit_price(), 45.0);
    }

    #[test]
    fn unit_price_of_a_zero_quantity_falls_back_to_the_total() {
        // parse_quantity rejects zero, so this only guards against hand-edited JSON;
        // the fallback keeps it finite rather than poisoning every average with inf.
        let broken = purchase(1, "Oeufs", "Lidl", 240, 0.0, 1);
        assert_eq!(broken.unit_price(), 240.0);
    }

    #[test]
    fn parses_both_decimal_separators_and_a_stray_currency_symbol() {
        assert_eq!(parse_price_to_cents("2,45"), Some(245));
        assert_eq!(parse_price_to_cents("2.45"), Some(245));
        assert_eq!(parse_price_to_cents("2"), Some(200));
        assert_eq!(parse_price_to_cents(" 2,45 € "), Some(245));
        assert_eq!(parse_price_to_cents("0,05"), Some(5));
    }

    #[test]
    fn refuses_prices_and_quantities_that_would_corrupt_a_comparison() {
        assert_eq!(parse_price_to_cents(""), None);
        assert_eq!(parse_price_to_cents("gratuit"), None);
        assert_eq!(parse_price_to_cents("-1"), None);
        assert_eq!(parse_quantity("0"), None);
        assert_eq!(parse_quantity("-2"), None);
        assert_eq!(parse_quantity("abc"), None);
        assert_eq!(parse_quantity("1,5"), Some(1.5));
    }

    #[test]
    fn formats_money_with_two_decimals_and_thin_unit_prices_with_three() {
        assert_eq!(format_cents(1234), "12,34 €");
        assert_eq!(format_cents(5), "0,05 €");
        assert_eq!(format_cents(-250), "-2,50 €");
        assert_eq!(format_unit_price(245.0, Unit::Kg), "2,45 €/kg");
        // Comma here too — a screen mixing 2,45 € with 0.040 € reads as two apps.
        assert_eq!(format_unit_price(4.0, Unit::Piece), "0,040 €/u");
    }

    #[test]
    fn grams_and_millilitres_are_converted_on_entry_not_stored() {
        // A 250 g packet and a 1 kg block have to end up comparable.
        assert_eq!(InputUnit::Gram.to_canonical(250.0), (Unit::Kg, 0.25));
        assert_eq!(InputUnit::Kg.to_canonical(1.0), (Unit::Kg, 1.0));
        assert_eq!(
            InputUnit::Milliliter.to_canonical(500.0),
            (Unit::Liter, 0.5)
        );
        assert_eq!(InputUnit::Piece.to_canonical(6.0), (Unit::Piece, 6.0));

        let mut db = Db::default();
        let (unit, quantity) = InputUnit::Gram.to_canonical(250.0);
        let mut packet = purchase(1, "Beurre", "Lidl", 245, quantity, 1);
        packet.unit = unit;
        db.insert(packet);
        // 2,45 € for 250 g is 9,80 € a kilo, and that is the number to compare.
        assert_eq!(db.summaries(None)[0].latest.unit_price(), 980.0);
    }

    #[test]
    fn quantities_read_back_the_way_a_packet_is_labelled() {
        assert_eq!(format_quantity(0.25, Unit::Kg), "250 g");
        assert_eq!(format_quantity(1.5, Unit::Kg), "1,5 kg");
        assert_eq!(format_quantity(0.5, Unit::Liter), "500 ml");
        assert_eq!(format_quantity(2.0, Unit::Liter), "2 L");
        assert_eq!(format_quantity(6.0, Unit::Piece), "6");
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(0.75), "0,75");
    }

    #[test]
    fn one_product_is_one_row_whatever_its_spelling() {
        let mut db = Db::default();
        db.insert(purchase(1, "Lait  demi", "Lidl", 100, 1.0, 1));
        db.insert(purchase(2, "lait demi", "LIDL", 110, 1.0, 2));
        let summaries = db.summaries(None);
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].purchase_count, 2);
        assert_eq!(summaries[0].stores.len(), 1);
        // Display keeps the most recent spelling, not the first one.
        assert_eq!(summaries[0].product, "lait demi");
    }

    #[test]
    fn cheapest_store_ranks_on_the_average_not_a_single_promotion() {
        let mut db = Db::default();
        // Aldi: one 1,00 € promotion then 3,00 € twice -> mean 2,33.
        db.insert(purchase(1, "Beurre", "Aldi", 100, 1.0, 1));
        db.insert(purchase(2, "Beurre", "Aldi", 300, 1.0, 2));
        db.insert(purchase(3, "Beurre", "Aldi", 300, 1.0, 3));
        // Lidl: steady 2,00 €.
        db.insert(purchase(4, "Beurre", "Lidl", 200, 1.0, 4));
        let summary = &db.summaries(None)[0];
        assert_eq!(summary.best().unwrap().store, "Lidl");
        assert!((summary.savings_per_unit().unwrap() - 33.333).abs() < 0.01);
    }

    #[test]
    fn mixing_units_never_averages_kilos_with_items() {
        let mut db = Db::default();
        db.insert(purchase(1, "Tomates", "Lidl", 400, 1.0, 1));
        let mut by_piece = purchase(2, "Tomates", "Aldi", 50, 1.0, 2);
        by_piece.unit = Unit::Piece;
        db.insert(by_piece);
        let summary = &db.summaries(None)[0];
        // Latest purchase is the per-item one, so only per-item lines are compared.
        assert_eq!(summary.unit, Unit::Piece);
        assert_eq!(summary.stores.len(), 1);
        assert_eq!(summary.stores[0].store, "Aldi");
        // The kilo line is still in the history, just not in the comparison.
        assert_eq!(summary.purchase_count, 2);
    }

    #[test]
    fn trend_ignores_a_rounding_sized_difference() {
        let mut db = Db::default();
        db.insert(purchase(1, "Pain", "Lidl", 100, 1.0, 1));
        db.insert(purchase(2, "Pain", "Lidl", 200, 1.0, 2));
        assert_eq!(db.summaries(None)[0].trend(), Trend::Up);

        let mut db = Db::default();
        db.insert(purchase(1, "Pain", "Lidl", 200, 1.0, 1));
        db.insert(purchase(2, "Pain", "Lidl", 100, 1.0, 2));
        assert_eq!(db.summaries(None)[0].trend(), Trend::Down);

        let mut db = Db::default();
        db.insert(purchase(1, "Pain", "Lidl", 200, 1.0, 1));
        db.insert(purchase(2, "Pain", "Lidl", 201, 1.0, 2));
        assert_eq!(db.summaries(None)[0].trend(), Trend::Flat);

        let mut db = Db::default();
        db.insert(purchase(1, "Pain", "Lidl", 200, 1.0, 1));
        assert_eq!(db.summaries(None)[0].trend(), Trend::Unknown);
    }

    #[test]
    fn store_filter_narrows_the_rows_but_not_the_comparison() {
        let mut db = Db::default();
        db.insert(purchase(1, "Beurre", "Aldi", 150, 1.0, 1));
        db.insert(purchase(2, "Beurre", "Lidl", 200, 1.0, 2));
        db.insert(purchase(3, "Cafe", "Lidl", 500, 1.0, 3));

        let lidl = db.summaries(Some("lidl"));
        assert_eq!(lidl.len(), 2);
        let beurre = lidl.iter().find(|s| s.product == "Beurre").unwrap();
        // Latest *at Lidl*, ...
        assert_eq!(beurre.latest.price_cents, 200);
        // ... but Aldi is still named as the cheaper option.
        assert_eq!(beurre.best().unwrap().store, "Aldi");

        let aldi = db.summaries(Some("Aldi"));
        assert_eq!(aldi.len(), 1);
        assert_eq!(aldi[0].product, "Beurre");
    }

    #[test]
    fn the_plan_groups_products_under_their_cheapest_store() {
        let mut db = Db::default();
        db.insert(purchase(1, "Beurre", "Aldi", 150, 1.0, 1));
        db.insert(purchase(2, "Beurre", "Lidl", 200, 1.0, 2));
        db.insert(purchase(3, "Cafe", "Lidl", 500, 1.0, 3));
        db.insert(purchase(4, "Cafe", "Aldi", 900, 1.0, 4));

        let plan = db.best_store_plan();
        assert_eq!(plan.len(), 2);
        // Coffee saves 4,00 € a kilo at Lidl, butter only 0,50 € at Aldi.
        assert_eq!(plan[0].0, "Lidl");
        assert_eq!(plan[0].1[0].product, "Cafe");
        assert_eq!(plan[1].0, "Aldi");
        assert_eq!(plan[1].1[0].product, "Beurre");
    }

    #[test]
    fn suggestions_put_the_most_recently_used_names_first() {
        let mut db = Db::default();
        db.insert(purchase(1, "Beurre", "Aldi", 150, 1.0, 1));
        db.insert(purchase(2, "Cafe", "Lidl", 500, 1.0, 5));
        assert_eq!(db.stores(), vec!["Lidl", "Aldi"]);
        assert_eq!(db.product_names(), vec!["Cafe", "Beurre"]);
    }

    #[test]
    fn ids_stay_unique_among_the_rows_that_still_exist() {
        let mut db = Db::default();
        db.insert(purchase(1, "Beurre", "Aldi", 150, 1.0, 1));
        db.insert(purchase(2, "Cafe", "Lidl", 500, 1.0, 2));
        assert_eq!(db.next_id(), 3);

        // Deleting the highest id frees it for reuse. That is fine here — an id is
        // only ever a list key and a delete target within one snapshot, and nothing
        // holds a reference to a deleted row — but it does mean an id is not a
        // durable identity, so nothing should ever be stored against one.
        db.remove(2);
        assert_eq!(db.next_id(), 2);
    }

    #[test]
    fn product_key_ignores_spelling_and_survives_slashes() {
        assert_eq!(product_key("Lait demi"), product_key("  lait   DEMI "));
        assert_ne!(product_key("Lait"), product_key("Pain"));

        let mut db = Db::default();
        db.insert(purchase(1, "Lait 1/2 écrémé", "Lidl", 100, 1.0, 1));
        let key = product_key("Lait 1/2 écrémé");
        assert_eq!(db.product_by_key(key).as_deref(), Some("Lait 1/2 écrémé"));
        assert_eq!(db.product_by_key(product_key("Absent")), None);
    }

    #[test]
    fn a_stored_database_survives_a_round_trip() {
        let mut db = Db::default();
        db.insert(purchase(1, "Beurre", "Aldi", 150, 0.25, 1));
        let json = serde_json::to_string(&db).unwrap();
        assert_eq!(serde_json::from_str::<Db>(&json).unwrap(), db);
    }
}
