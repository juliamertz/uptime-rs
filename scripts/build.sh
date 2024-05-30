#!/usr/bin/env bash

bash ./scripts/tailwind.sh
echo "Building cargo project..."
cargo build --release
