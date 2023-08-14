#!/bin/bash

set -e

umask 0000

cargo clean --release
cargo build --target x86_64-unknown-linux-gnu --release --bin luna_deny_cakes_game_sdl --features sdl

cp /usr/local/lib/{libSDL2-2.0.so.0,libSDL2_image-2.0.so.0,libSDL2_mixer-2.0.so.0,libSDL2_ttf-2.0.so.0} \
    target/x86_64-unknown-linux-gnu/release
