//! Where the data lives: `localStorage` in the browser, a JSON file on disk on
//! desktop and Android. Nothing ever leaves the device — there is no server, and
//! no network code in this crate at all.

use dioxus::prelude::*;
use dioxus_sdk_storage::LocalStorage;

use crate::model::{Db, ProductSummary};

/// Storage key, and on native the literal filename under the app data directory.
/// Versioned so a future breaking change to [`Db`] can migrate rather than eat
/// the existing history.
pub const DB_KEY: &str = "panier-db-v1";

/// UI language, `"fr"` or `"en"`.
pub const LANG_KEY: &str = "panier-lang";

/// Newtype wrapper: Dioxus keys context by `TypeId`, so a bare `Signal<Db>` would
/// collide with any other signal of the same type provided anywhere in the tree.
#[derive(Clone, Copy)]
pub struct CtxDb(pub Signal<Db>);

#[derive(Clone, Copy)]
pub struct CtxLang(pub Signal<String>);

/// Points the filesystem backend at the app's data directory. Must run before any
/// storage hook: the backend panics rather than guessing a location.
#[cfg(not(target_arch = "wasm32"))]
pub fn init_native_dir() {
    // Android has no per-OS data directory to resolve — the `directories` crate
    // behind `set_dir!()` has no Android support and panics there — so it gets the
    // path spelled out. It must match [bundle].identifier in Dioxus.toml, or the
    // installed app reads a directory nothing ever wrote to.
    #[cfg(target_os = "android")]
    dioxus_sdk_storage::set_dir!("/data/data/io.github.r0ndoudou.panier/files/panier");

    // `set_dir_name`, not the bare `set_dir!()`: that macro names the folder after
    // `CARGO_PKG_NAME`, which would put a user's backup under `dx-panier/` — the
    // crate name, which means nothing to them. Naming it here keeps the folder
    // "panier" on every desktop OS, matching the app and the Android path above.
    #[cfg(not(target_os = "android"))]
    dioxus_sdk_storage::set_dir_name("panier");
}

/// Creates the two persisted signals, publishes them to the tree, and hands the
/// language one back for `App` to drive dioxus-i18n with.
///
/// Called once, from `App`, and never from a `#[layout]` component: a
/// `use_synced_storage` call inside a layout re-runs on every navigation and
/// overflows the stack.
pub fn use_provide_storage() -> Signal<String> {
    let db =
        dioxus_sdk_storage::use_synced_storage::<LocalStorage, Db>(DB_KEY.to_owned(), Db::default);
    let lang =
        dioxus_sdk_storage::use_synced_storage::<LocalStorage, String>(LANG_KEY.to_owned(), || {
            "fr".to_owned()
        });
    use_context_provider(|| CtxDb(db));
    use_context_provider(|| CtxLang(lang));
    lang
}

/// The purchase database. Writing through the returned signal persists it.
pub fn use_db() -> Signal<Db> {
    use_context::<CtxDb>().0
}

pub fn use_lang() -> Signal<String> {
    use_context::<CtxLang>().0
}

/// Values every page wants but none should rebuild: the product and store
/// name lists, and the per-product comparison behind Produits and Où acheter.
///
/// Each is a full walk over every purchase, and each page used to redo them on
/// mount — so switching tabs paid for them again every time. Provided from
/// `Shell`, which is the router layout and therefore stays mounted across
/// navigations, these are computed once and recomputed only when the database
/// actually changes.
#[derive(Clone, Copy)]
pub struct CtxDerived {
    pub product_names: Memo<Vec<String>>,
    pub stores: Memo<Vec<String>>,
    pub summaries: Memo<Vec<ProductSummary>>,
}

pub fn use_provide_derived() {
    let db = use_db();
    let product_names = use_memo(move || db.read().product_names());
    let stores = use_memo(move || db.read().stores());
    let summaries = use_memo(move || db.read().summaries(None));
    use_context_provider(|| CtxDerived {
        product_names,
        stores,
        summaries,
    });
}

pub fn use_derived() -> CtxDerived {
    use_context::<CtxDerived>()
}
