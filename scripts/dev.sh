#!/usr/bin/env bash

./scripts/tailwind.sh --watch &
tailwind_pid=$!
# echo "Tailwind PID: $tailwind_pid"
# echo "Running cargo project..."
cargo watch --exec run
# kill $tailwind_pid
