#!/bin/bash
# Runs the web client with hot reload.
# IP=0.0.0.0 makes it reachable from other devices on the LAN, so a phone browser
# can hit http://<this-machine>:8080 without building the APK at all.
set -euo pipefail
IP=0.0.0.0 dx serve --platform web
