#!/bin/sh

set -e

DEST=dist/windows
mkdir -pv "$DEST"

cp -pv target/x86_64-pc-windows-gnu/release/luna_deny_cakes_game_{macroquad,sdl,sfml}.exe "$DEST"
cp -pv target/x86_64-pc-windows-gnu/release/*.dll "$DEST"

cp -Rpv data "$DEST"

chmod -R a+rX "$DEST"

echo Done
