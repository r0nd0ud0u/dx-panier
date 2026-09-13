#!/bin/bash
# Bundles the Android client as a sideloadable APK.
#
# --package-types apk forces a directly-installable .apk instead of dx's default
# .aab (Play Store bundle format), which a phone cannot install on its own.
#
# Usage: ./scripts/bundle_mobile.sh [rustc-target-triple]
# The triple must be explicit: dx's own default may not match the device, giving an
# APK that fails to install ("app not compatible with this device"). Defaults to
# aarch64-linux-android (every real phone since ~2019); x86_64-linux-android works
# for emulators. 32-bit targets do not work at all — dioxus 0.7's manganis errors
# with "Only 64-bit Android targets are supported".
set -euo pipefail
TARGET="${1:-aarch64-linux-android}"
dx bundle --platform android --release --package-types apk --target "$TARGET" \
  --no-default-features --features mobile --out-dir bundle-android
