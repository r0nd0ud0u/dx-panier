#!/bin/bash
# Runs the desktop client with hot reload.
set -euo pipefail
dx serve --platform desktop --no-default-features --features desktop
