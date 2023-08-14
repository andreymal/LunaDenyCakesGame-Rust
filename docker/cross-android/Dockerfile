FROM notfl3/cargo-apk@sha256:72eb4c2201448377f4b2fe137e3d0078938a7d6dc021446b38127282653c5a06

RUN apt-get update && \
    apt-get -y install git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY cargo-quad-apk.patch /tmp

# Install patched cargo-quad-apk

RUN cd /tmp && \
    git clone https://github.com/not-fl3/cargo-quad-apk.git && \
    cd cargo-quad-apk && \
    git reset --hard 2f5657f && \
    git apply /tmp/cargo-quad-apk.patch && \
    cargo install --path . && \
    cd / && \
    rm -rf /tmp/cargo-quad-apk /tmp/cargo-quad-apk.patch

# Install build script

WORKDIR /root/src

COPY --chmod=755 build.sh build-nosign.sh /usr/local/bin/
CMD build.sh
