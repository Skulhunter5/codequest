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

COPY . .
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=workspace-target,target=/workspace/target \
    cargo build --release \
        --bin codequest-bootstrap \
        --bin codequest-gateway \
        --bin codequest-user-service \
        --bin codequest-quest-service \
        --bin codequest-progression-service \
    && cp /workspace/target/release/codequest-{bootstrap,gateway,{user,quest,progression}-service} /workspace/

# BINARY: codequest-bootstrap
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/codequest-bootstrap /usr/local/bin/codequest-bootstrap
ENTRYPOINT ["codequest-bootstrap"]
LABEL service=bootstrap

# BINARY: codequest-gateway
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/static/* /app/static/
COPY --from=builder /workspace/codequest-gateway /usr/local/bin/codequest-gateway
ENTRYPOINT ["codequest-gateway"]
LABEL service=gateway

# BINARY: codequest-user-service
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/codequest-user-service /usr/local/bin/codequest-user-service
ENTRYPOINT ["codequest-user-service"]
LABEL service=users

# BINARY: codequest-quest-service
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/codequest-quest-service /usr/local/bin/codequest-quest-service
ENTRYPOINT ["codequest-quest-service"]
LABEL service=quests

# BINARY: codequest-progression-service
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/codequest-progression-service /usr/local/bin/codequest-progression-service
ENTRYPOINT ["codequest-progression-service"]
LABEL service=progression
