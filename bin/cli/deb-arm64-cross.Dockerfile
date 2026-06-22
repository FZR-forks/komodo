## Cross-compile Komodo CLI for ARM64 and package as DEB
## Usage: docker buildx build -f bin/cli/deb-arm64-cross.Dockerfile -t komodo-cli-deb-arm64 --output type=local,dest=./output .

ARG VERSION=2.0.0

FROM rust:1 AS builder

RUN dpkg --add-architecture arm64 && apt-get update && apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    pkg-config \
    libssl-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

WORKDIR /builder

COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/cli ./bin/cli

RUN cargo build -p komodo_cli --release --target aarch64-unknown-linux-gnu

FROM debian:stable AS debbuilder
ARG VERSION

RUN apt-get update && apt-get install -y --no-install-recommends \
    dpkg-dev \
    dpkg \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /builder/target/aarch64-unknown-linux-gnu/release/km /tmp/km

RUN mkdir -p /tmp/debroot/usr/bin /tmp/debroot/DEBIAN && \
    install -m 755 /tmp/km /tmp/debroot/usr/bin/km && \
    printf 'Package: komodo-cli\nVersion: %s\nSection: utils\nPriority: optional\nArchitecture: arm64\nMaintainer: mbecker20 <becker.maxh@gmail.com>\nDescription: Komodo CLI\n Command line tool for Komodo.\n' "$VERSION" > /tmp/debroot/DEBIAN/control && \
    chmod 0644 /tmp/debroot/DEBIAN/control && \
    dpkg-deb --build /tmp/debroot "/tmp/komodo-cli_${VERSION}_arm64.deb"

FROM scratch
ARG VERSION
COPY --from=debbuilder "/tmp/komodo-cli_${VERSION}_arm64.deb" /
