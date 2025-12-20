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

ENV BINARY=codequest-progression-service
COPY . .
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=workspace-target,target=/workspace/target \
    cargo build --release -p $BINARY \
    && cp /workspace/target/release/$BINARY /workspace/$BINARY

# BINARY
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/codequest-progression-service /usr/local/bin/codequest-progression-service
ENTRYPOINT ["codequest-progression-service"]
