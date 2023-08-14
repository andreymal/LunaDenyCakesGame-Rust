#!/bin/bash

set -e

umask 0000

cargo clean --release
cargo build --target x86_64-pc-windows-gnu --release --bin luna_deny_cakes_game_sfml --features sfml

cp /opt/SFML-2.6.0/bin/{openal32,sfml-audio-2,sfml-graphics-2,sfml-system-2,sfml-window-2}.dll \
    target/x86_64-pc-windows-gnu/release

cp /usr/lib/gcc/x86_64-w64-mingw32/12-win32/{libstdc++-6,libgcc_s_seh-1}.dll \
    target/x86_64-pc-windows-gnu/release
