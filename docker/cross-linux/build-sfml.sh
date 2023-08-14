#!/bin/bash

set -e

umask 0000

cargo clean --release
cargo build --target x86_64-unknown-linux-gnu --release --bin luna_deny_cakes_game_sfml --features sfml

cp /usr/local/lib/libfreetype.so.6 \
    target/x86_64-unknown-linux-gnu/release

cp /usr/local/lib64/{libFLAC.so.12,libogg.so.0,libopenal.so.1,libvorbisenc.so.2.0.12,libvorbisfile.so.3.3.8,libvorbis.so.0.4.9} \
    target/x86_64-unknown-linux-gnu/release

cp /usr/local/lib64/{libsfml-audio.so.2.6,libsfml-graphics.so.2.6,libsfml-system.so.2.6,libsfml-window.so.2.6} \
    target/x86_64-unknown-linux-gnu/release
