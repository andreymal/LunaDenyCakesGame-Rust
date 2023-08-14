FROM debian:bookworm

RUN \
    apt-get update && \
    apt-get -y install --no-install-recommends ca-certificates curl gcc libc-dev make mingw-w64 unzip && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir /root/src /usr/local/i686-w64-mingw32 /usr/local/x86_64-w64-mingw32

# Install Rust

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN \
    curl -sSLf -o /tmp/rustup-init.sh https://sh.rustup.rs && \
    chmod +x /tmp/rustup-init.sh && \
    /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable && \
    rm /tmp/rustup-init.sh && \
    chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME" && \
    rustup target add x86_64-pc-windows-gnu

# Install SDL

RUN \
    cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL/releases/download/release-2.28.2/SDL2-devel-2.28.2-mingw.tar.gz | tar -xvzf - && \
    cd SDL2-2.28.2 && \
    make CROSS_PATH=/usr cross && \
    cd / && rm -rf /tmp/SDL2-2.28.2

RUN \
    cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL_image/releases/download/release-2.6.3/SDL2_image-devel-2.6.3-mingw.tar.gz | tar -xvzf - && \
    cd SDL2_image-2.6.3 && \
    make CROSS_PATH=/usr cross && \
    cd / && rm -rf /tmp/SDL2_image-2.6.3

RUN \
    cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL_mixer/releases/download/release-2.6.3/SDL2_mixer-devel-2.6.3-mingw.tar.gz | tar -xvzf - && \
    cd SDL2_mixer-2.6.3 && \
    make CROSS_PATH=/usr cross && \
    cd / && rm -rf /tmp/SDL2_mixer-2.6.3

RUN \
    cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL_ttf/releases/download/release-2.20.2/SDL2_ttf-devel-2.20.2-mingw.tar.gz | tar -xvzf - && \
    cd SDL2_ttf-2.20.2 && \
    make CROSS_PATH=/usr cross && \
    cd / && rm -rf /tmp/SDL2_ttf-2.20.2

# Install SFML

RUN \
    curl -sSLf -o /tmp/SFML.zip https://www.sfml-dev.org/files/SFML-2.6.0-windows-gcc-13.1.0-mingw-64-bit.zip && \
    cd /opt && \
    unzip /tmp/SFML.zip && \
    rm /tmp/SFML.zip

ENV SFML_INCLUDE_DIR=/opt/SFML-2.6.0/include \
    SFML_LIBS_DIR=/opt/SFML-2.6.0/lib

# Install build scripts

WORKDIR /root/src

COPY --chmod=755 build-all.sh build-macroquad.sh build-sdl.sh build-sfml.sh /usr/local/bin/
CMD build-all.sh
