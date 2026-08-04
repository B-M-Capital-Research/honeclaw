# syntax=docker/dockerfile:1.7

# Production is Debian Linux on x86_64. Building on the older Bookworm ABI
# keeps the extracted binaries compatible with the newer production host.
FROM --platform=linux/amd64 rust:1.95.0-bookworm@sha256:4c2fd73ef19c5ef9d54bee03b06b2839a392604fbfcd578ed948b71b37c1d7fb AS chef

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        clang \
        cmake \
        git \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

RUN --mount=type=cache,id=hone-runtime-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=hone-runtime-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    cargo install --locked --version 0.1.77 cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /workspace/recipe.json recipe.json

# This dependency-only layer is exported by BuildKit's GHA cache. Unlike a
# cache mount, it survives ephemeral GitHub-hosted runners when source changes.
RUN --mount=type=cache,id=hone-runtime-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=hone-runtime-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --locked --profile source-runtime \
        --recipe-path recipe.json \
        --package hone-cli \
        --package hone-console-page \
        --package hone-discord \
        --package hone-feishu \
        --package hone-mcp \
        --package hone-telegram

COPY . .

ARG HONE_BUILD_GIT_SHA
ARG HONE_BUILD_TIMESTAMP
ARG HONE_BUILD_SOURCE=ghcr_linux_oci

RUN test "${#HONE_BUILD_GIT_SHA}" -eq 40 \
    && test -n "${HONE_BUILD_TIMESTAMP}" \
    && test "${HONE_BUILD_SOURCE}" = "ghcr_linux_oci"

RUN --mount=type=cache,id=hone-runtime-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=hone-runtime-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    HONE_BUILD_GIT_SHA="${HONE_BUILD_GIT_SHA}" \
    HONE_BUILD_TIMESTAMP="${HONE_BUILD_TIMESTAMP}" \
    HONE_BUILD_SOURCE="${HONE_BUILD_SOURCE}" \
    cargo build --locked --profile source-runtime \
        --package hone-cli \
        --package hone-console-page \
        --package hone-discord \
        --package hone-feishu \
        --package hone-mcp \
        --package hone-telegram \
    && HONE_BUILD_GIT_SHA="${HONE_BUILD_GIT_SHA}" \
       HONE_BUILD_TIMESTAMP="${HONE_BUILD_TIMESTAMP}" \
       HONE_BUILD_SOURCE="${HONE_BUILD_SOURCE}" \
       HONE_BUILD_PROFILE=source-runtime \
       HONE_BUILD_TARGET=x86_64-unknown-linux-gnu \
       bash scripts/package_runtime_bundle.sh \
           /workspace/target/source-runtime \
           /out/release

FROM scratch

ARG HONE_BUILD_GIT_SHA
ARG HONE_BUILD_TIMESTAMP

LABEL org.opencontainers.image.title="HONE Linux runtime bundle" \
      org.opencontainers.image.description="Immutable HONE production runtime extracted by the GCE systemd deployment" \
      org.opencontainers.image.source="https://github.com/B-M-Capital-Research/honeclaw" \
      org.opencontainers.image.revision="${HONE_BUILD_GIT_SHA}" \
      org.opencontainers.image.created="${HONE_BUILD_TIMESTAMP}"

COPY --from=builder /out/release /release
