FROM quay.io/pypa/manylinux2014_x86_64

# Без ulimit rpm зависает, офигевая от количества доступных файловых дескрипторов
RUN \
    ulimit -n 1024 && \
    yum update && \
    yum install -y alsa-lib-devel pulseaudio-libs-devel systemd-devel libXcursor-devel libXrandr-devel && \
    rm -rf /var/cache/yum

# Install Rust

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN \
    curl -sSLf -o /tmp/rustup-init.sh https://sh.rustup.rs && \
    chmod +x /tmp/rustup-init.sh && \
    /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable && \
    rm /tmp/rustup-init.sh && \
    chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME"

# Install SDL

RUN cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL/releases/download/release-2.28.2/SDL2-2.28.2.tar.gz | tar -xzf - && \
    cd SDL2-2.28.2 && \
    mkdir build && cd build && \
    ../configure && \
    make && \
    make install && \
    cd / && \
    rm -rf /tmp/SDL2-2.28.2

RUN cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL_image/releases/download/release-2.6.3/SDL2_image-2.6.3.tar.gz | tar -xzf - && \
    cd SDL2_image-2.6.3 && \
    mkdir build && cd build && \
    ../configure && \
    make && \
    make install && \
    cd / && \
    rm -rf /tmp/SDL2_image-2.6.3

RUN cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL_mixer/releases/download/release-2.6.3/SDL2_mixer-2.6.3.tar.gz | tar -xzf - && \
    cd SDL2_mixer-2.6.3 && \
    mkdir build && cd build && \
    ../configure && \
    make && \
    make install && \
    cd / && \
    rm -rf /tmp/SDL2_mixer-2.6.3

RUN cd /tmp && \
    curl -sSLf https://github.com/libsdl-org/SDL_ttf/releases/download/release-2.20.2/SDL2_ttf-2.20.2.tar.gz | tar -xzf - && \
    cd SDL2_ttf-2.20.2 && \
    mkdir build && cd build && \
    ../configure && \
    make && \
    make install && \
    cd / && \
    rm -rf /tmp/SDL2_ttf-2.20.2

# Install SFML dependencies

RUN cd /tmp && \
    curl -sSLf https://download.savannah.gnu.org/releases/freetype/freetype-2.13.1.tar.xz | tar -xJf - && \
    cd freetype-2.13.1 && \
    mkdir build && cd build && \
    ../configure --with-zlib=no && \
    make && \
    make install && \
    cd / && \
    rm -rf /tmp/freetype-2.13.1 && \
    ln -s /usr/local/include/freetype2/* /usr/local/include/  # ?????

RUN cd /tmp && \
    curl -sSLf https://openal-soft.org/openal-releases/openal-soft-1.23.1.tar.bz2 | tar -xjf - && \
    cd openal-soft-1.23.1 && \
    mkdir -p build && cd build && \
    cmake .. && \
    cmake --build . && \
    cmake --install . && \
    cd / && \
    rm -rf /tmp/openal-soft-1.23.1

RUN cd /tmp && \
    curl -ssLf https://ftp.osuosl.org/pub/xiph/releases/ogg/libogg-1.3.5.tar.xz | tar -xJf - && \
    cd libogg-1.3.5 && \
    mkdir -p build && cd build && \
    cmake -DBUILD_SHARED_LIBS=ON .. && \
    cmake --build . && \
    cmake --install . && \
    cd / && \
    rm -rf /tmp/libogg-1.3.5

RUN cd /tmp && \
    curl -ssLf https://ftp.osuosl.org/pub/xiph/releases/vorbis/libvorbis-1.3.7.tar.xz | tar -xJf - && \
    cd libvorbis-1.3.7 && \
    mkdir -p build && cd build && \
    cmake -DBUILD_SHARED_LIBS=ON .. && \
    cmake --build . && \
    cmake --install . && \
    cd / && \
    rm -rf /tmp/libvorbis-1.3.7

RUN cd /tmp && \
    curl -ssLf https://ftp.osuosl.org/pub/xiph/releases/flac/flac-1.4.3.tar.xz | tar -xJf - && \
    cd flac-1.4.3 && \
    mkdir -p build && cd build && \
    cmake -DBUILD_SHARED_LIBS=ON .. && \
    cmake --build . && \
    cmake --install . && \
    cd / && \
    rm -rf /tmp/flac-1.4.3

# Install SFML

RUN cd /tmp && \
    curl -sSLfO https://www.sfml-dev.org/files/SFML-2.6.0-sources.zip && \
    unzip SFML-2.6.0-sources.zip && \
    cd SFML-2.6.0 && \
    mkdir build && cd build && \
    cmake .. && \
    cmake --build . && \
    cmake --install . && \
    cd / && \
    rm -rf /tmp/SFML-2.6.0 /tmp/SFML-2.6.0-sources.zip

# Install build scripts

WORKDIR /root/src

COPY --chmod=755 build-all.sh build-macroquad.sh build-sdl.sh build-sfml.sh /usr/local/bin/
CMD build-all.sh
