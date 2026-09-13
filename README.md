# Panier

Grocery price tracking, product by product and store by store. You record what you
paid on the way out of the store; the app answers the question "where should I buy
what".

Everything is stored **on the device**. There is no server, no account, no network:
not a single line of code in this application opens a connection.

Written in Rust with [Dioxus](https://dioxuslabs.com/); the same codebase produces
an Android APK, a desktop executable (Linux, Windows) and a static website.

## What it does

- **Add** a purchase: product, store, price paid, quantity, unit, date. Names you
  have already entered are offered as autocompletion; the store and the date carry
  over from one entry to the next, because a receipt is ten products from a single
  store on the same day.
- **Unit price**: everything is reduced to a price per kilo, per litre or per item.
  That is what makes a 250 g pack and a 500 g loaf comparable. Quantities can be
  entered in g or ml if that is what the packaging says — the conversion happens
  when the entry is saved.
- **Products**: the list of everything that has been bought, with the latest price,
  the trend, and the cheapest store. Filterable by store.
- **Product detail**: the unit price curve over time, the store-by-store comparison,
  and the full history.
- **Where to buy**: every product filed under the store that sells it cheapest on
  average, with the difference per unit.
- **Settings**: French/English, JSON backup and restore, wipe.

## Development

```bash
./scripts/dev_web.sh        # browser, with hot reload
./scripts/dev_desktop.sh    # native window
./scripts/dev_android.sh    # phone or emulator connected over USB
```

`dev_web.sh` listens on `0.0.0.0`, so a phone can also just open
`http://<machine-ip>:8080` without installing anything.

Requirements: stable Rust and `dioxus-cli` 0.7.10 (`cargo binstall dioxus-cli@0.7.10`).
For Android, see [docs/android-local.md](docs/android-local.md).

## Checks

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The tests cover the computation: unit price, unit conversions, store ranking,
trends, filters, and parity between the two translation files.

## Installing the APK on your phone

Pushing a tag triggers CI, which builds the binaries and attaches them to the
GitHub release:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

The APK shows up as a release asset named `panier-v0.1.0-arm64-v8a.apk`. It can be
downloaded straight from the phone; Android will ask you to allow installation from
that source.

The APK is signed with `android/debug.keystore`, which is committed **on purpose**:
without a stable key, Android refuses to install a new version over the old one.
This is not a secret — it is the standard debug key, which anyone can regenerate.
It is not suitable for publishing on the Play Store.

To build the APK locally rather than in CI, see
[docs/android-local.md](docs/android-local.md).

## Where the data lives

| Platform | Location |
| --- | --- |
| Browser | `localStorage`, key `panier-db-v1` |
| Linux | `~/.local/share/panier/panier-db-v1` |
| Windows | `%LOCALAPPDATA%\panier\panier-db-v1` |
| Android | `/data/data/io.github.r0ndoudou.panier/files/panier/panier-db-v1` |

These locations do not talk to each other: a phone and a browser each have their
own history. The settings let you export JSON from one and restore it in the other.

Uninstalling the application erases its data. **Export first.**

## Layout

```
src/model.rs     the whole domain: purchases, unit prices, comparisons (and their tests)
src/storage.rs   local persistence, a single JSON key
src/common.rs    routes and first-render constants
src/pages/       one page per tab, plus the shared widgets
scripts/         launching and packaging for the three platforms
```

## License

Apache-2.0.
