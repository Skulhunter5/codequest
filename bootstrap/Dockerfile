# CHEF
FROM rust:1.92 AS chef

WORKDIR /workspace
RUN cargo install cargo-chef --locked

# PLANNER
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# BUILDER
FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=workspace-target,target=/workspace/target \
    cargo chef cook --release --recipe-path recipe.json

ENV BINARY=codequest-bootstrap
COPY . .
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=workspace-target,target=/workspace/target \
    cargo build --release -p $BINARY \
    && cp /workspace/target/release/$BINARY /workspace/$BINARY

# BINARY
FROM gcr.io/distroless/cc
WORKDIR /app
# COPY --from=builder /workspace/target/release/codequest-bootstrap /usr/local/bin/codequest-bootstrap
COPY --from=builder /workspace/codequest-bootstrap /usr/local/bin/codequest-bootstrap
# RUN chown -R nonroot:nonroot /app
# USER nonroot:nonroot
ENTRYPOINT ["codequest-bootstrap"]
