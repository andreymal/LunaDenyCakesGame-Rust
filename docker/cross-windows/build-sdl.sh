#!/bin/bash

set -e

umask 0000

cargo clean --release
cargo build --target x86_64-pc-windows-gnu --release --bin luna_deny_cakes_game_sdl --features sdl

cp /usr/x86_64-w64-mingw32/bin/{SDL2,SDL2_image,SDL2_mixer,SDL2_ttf}.dll \
    target/x86_64-pc-windows-gnu/release
