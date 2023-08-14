#!/bin/sh

set -e

DEST=dist/linux
mkdir -pv "$DEST"
mkdir -pv "$DEST"/bin
mkdir -pv "$DEST"/lib

cp -pv target/x86_64-unknown-linux-gnu/release/*.so.* "$DEST"/lib
cp -pv target/x86_64-unknown-linux-gnu/release/luna_deny_cakes_game_{macroquad,sdl,sfml} "$DEST"/bin
chmod 755 "$DEST"/bin/luna_deny_cakes_game_{macroquad,sdl,sfml}

for b in macroquad sdl sfml; do
    cat <<EOF >"$DEST/luna_deny_cakes_game_$b.sh"
#!/bin/sh

if [ -z "\$LD_LIBRARY_PATH" ]; then
    export LD_LIBRARY_PATH="\$PWD/lib"
else
    export LD_LIBRARY_PATH="\$LD_LIBRARY_PATH:\$PWD/lib"
fi

exec "\$PWD/bin/luna_deny_cakes_game_$b"
EOF
    chmod 755 "$DEST/luna_deny_cakes_game_$b.sh"
done

cp -Rpv data "$DEST"

chmod -R a+rX "$DEST"

echo Done
