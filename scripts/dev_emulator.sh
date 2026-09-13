#!/bin/bash
# Boots an Android emulator for local `dx serve --platform android` testing.
# Usage: ./scripts/dev_emulator.sh <avd-name> [--wipe] [--fresh]
# List available AVDs with: emulator -list-avds
#
# --wipe factory-resets the data partition: for INSTALL_FAILED_INSUFFICIENT_STORAGE,
# or a known-clean device. It also erases anything the app has saved.
# --fresh cold-boots instead of resuming the quick-boot snapshot: for a black or
# unresponsive window, or a device stuck 'offline'. Keeps apps and data.
AVD="${1:?Usage: $0 <avd-name> [--wipe] [--fresh]  (list available AVDs with: emulator -list-avds)}"
shift
EXTRA_ARGS=()
for arg in "$@"; do
    case "$arg" in
        --wipe) EXTRA_ARGS+=(-wipe-data) ;;
        --fresh) EXTRA_ARGS+=(-no-snapshot-load) ;;
        *) echo "unknown option: $arg (expected --wipe and/or --fresh)" >&2; exit 1 ;;
    esac
done
emulator -avd "$AVD" "${EXTRA_ARGS[@]}"
