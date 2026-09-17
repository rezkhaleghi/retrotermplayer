#!/bin/bash

set -e

SOURCE="https://www.youtube.com/watch?v=WvV5TbJc9tQ&list=RDWvV5TbJc9tQ&start_radio=1"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

open_terminal() {
    local MODE="$1"

    osascript <<EOF
tell application "Terminal"
    do script "cd '$PROJECT_DIR' && cargo run -- '$SOURCE' '$MODE'"
end tell
EOF
}

open_terminal 1
open_terminal 2
open_terminal 3
open_terminal 4