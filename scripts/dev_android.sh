#!/bin/bash
# Builds, installs and runs the Android client on a connected device or emulator.
#
# Needs ANDROID_HOME (SDK, build-tools, platform) and ANDROID_NDK_HOME — see
# docs/android-local.md. There is no `adb reverse` here and no SERVER_URL to set:
# the app talks to nothing, every byte it stores is on the device.
set -euo pipefail
dx serve --platform android --no-default-features --features mobile
