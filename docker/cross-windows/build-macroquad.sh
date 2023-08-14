#!/bin/bash

set -e

umask 0000

cargo clean --release
cargo build --target x86_64-pc-windows-gnu --release --bin luna_deny_cakes_game_macroquad --features macroquad
