#!/bin/bash
# Bundles the web client as a static site under bundle/public — no server process
# to run, so any static host (or `python3 -m http.server`) can serve it.
set -euo pipefail
dx bundle --platform web --release --out-dir bundle
