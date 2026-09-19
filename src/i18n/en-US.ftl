nav-add = Add
nav-products = Products
nav-plan = Where to buy
nav-settings = Settings

add-title = New purchase
field-product = Product
field-store = Store
field-price = Price paid
field-quantity = Quantity
field-unit = Unit
field-date = Date
field-note = Note
field-note-placeholder = Brand, special offer…
field-product-placeholder = Butter
field-store-placeholder = Lidl
field-piece-weight = Weight per piece (optional)

unit-kg = kilo
unit-g = gram
unit-l = litre
unit-ml = millilitre
unit-piece = item

action-add = Add
action-delete = Delete
action-cancel = Cancel
action-confirm = Confirm
action-import = Import
action-export = Export
action-back = Back

error-product = Enter a product.
error-store = Enter a store.
error-price = Invalid price — write 2.45, for example.
error-quantity = Invalid quantity — it must be greater than zero.
error-piece-weight = Invalid weight — leave this field empty if you don't know it.

recent-title = Latest entries
recent-empty = Nothing recorded yet.

products-title = Products
products-search = Search for a product
products-store-all = All stores
products-empty = Add a first purchase to see your products here.
products-no-match = No product matches.

label-best = Cheapest
label-purchases =
    { $count ->
        [one] { $count } purchase
       *[other] { $count } purchases
    }
label-since = since { $date }

trend-up = Going up
trend-down = Going down
trend-flat = Steady
trend-new = First entry

detail-not-found-title = Product not found
detail-not-found = This product no longer exists.
detail-evolution = Unit price over time
detail-by-store = Store by store
detail-history = History
detail-single-point = Only one entry so far — add another purchase to see the curve.
detail-mixed-units = Purchases in another unit stay in the history but are left out of the comparison.

plan-title = Where to buy what
plan-intro = Every product under the store that sells it cheapest on average.
plan-empty = Record the same product at two stores to get a comparison.
plan-savings = { $amount } gap per unit

settings-title = Settings
settings-language = Language
settings-data = Data
settings-stats = { $purchases } purchases · { $products } products · { $stores } stores
settings-export = Backup
# Android only — the file-export button isn't available there; see the
# comment on `BackupSection` (feature = "mobile") in settings.rs.
settings-export-help = Copy this text and keep it somewhere safe.
settings-backup-help = Save your purchases to a JSON file, and keep it somewhere safe.
settings-export-done = Backup exported.
settings-export-error = Couldn't save the file.
settings-import = Restore
# Android only, same limitation as settings-export-help above.
settings-import-help = Paste a backup here. It will replace the entire current history.
settings-restore-help = Choose a JSON backup file. It will replace the entire current history.
settings-import-error = Unreadable backup — nothing was changed.
settings-import-done =
    { $count ->
        [one] { $count } purchase restored.
       *[other] { $count } purchases restored.
    }
settings-wipe = Erase everything
settings-wipe-warning = The entire history will be permanently deleted.
settings-storage = Everything is stored on this device only. No data ever travels over the network.
