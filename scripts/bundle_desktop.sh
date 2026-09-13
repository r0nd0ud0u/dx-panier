#!/bin/bash
# Bundles the desktop client.
#
# rpm/nsis/msi all reject the hyphen in a pre-release version (e.g.
# "0.0.1-rc.0"), and dx has no flag to override the version, so Cargo.toml is
# temporarily rewritten to the numeric core for the bundle, then restored. The
# suffix still shows up in the tag, release, and asset filenames.
#
# Note: this makes an rc install as plain "0.0.1", same as the eventual stable
# release, per MSI's upgrade logic. Bump the core (0.0.2-rc.0) if that matters.
set -euo pipefail

version=$(grep -m1 '^version' Cargo.toml | sed -E 's/^version *= *"(.*)"/\1/')
core=${version%%-*}   # pre-release
core=${core%%+*}      # build metadata
if [ "$core" != "$version" ]; then
  echo "note: pre-release '$version' — packaging as '$core' (installers require numeric versions)"
  backup=$(mktemp)
  cp Cargo.toml "$backup"
  trap 'cp "$backup" Cargo.toml; rm -f "$backup"' EXIT INT TERM
  sed -i "0,/^version *= *\".*\"/s//version = \"$core\"/" Cargo.toml
fi

package_type_args=()
if [ "$(uname)" = "Linux" ]; then
  # Also emit .deb/.rpm alongside the default AppImage: unlike AppImage, they can
  # declare libwebkit2gtk-4.1-0 / webkit2gtk4.1 as a package dependency (see
  # Dioxus.toml [bundle.deb]), so apt/dnf installs it automatically.
  package_type_args=(--package-types appimage --package-types deb --package-types rpm)
fi
dx bundle --platform desktop --release --no-default-features --features desktop \
  --out-dir bundle-desktop "${package_type_args[@]}"
