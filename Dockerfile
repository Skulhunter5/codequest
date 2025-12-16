# BUILDER
FROM rust:1.92 AS chef

WORKDIR /workspace
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo install --path gateway
RUN cargo install --path user-service
RUN cargo install --path quest-service
RUN cargo install --path progression-service
# RUN cargo build --bin codequest-gateway --bin codequest-user-service --bin codequest-quest-service --bin codequest-progression-service

# codequest-gateway
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/static/* /app/static/
# COPY --from=builder /usr/local/cargo/bin/codequest-gateway /usr/local/bin/codequest-gateway
COPY --from=builder /workspace/target/release/codequest-gateway /usr/local/bin/codequest-gateway
# RUN chown -R nonroot:nonroot /app
# USER nonroot:nonroot
ENTRYPOINT ["codequest-gateway"]
LABEL service=gateway

# codequest-user-service
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
# COPY --from=builder /usr/local/cargo/bin/codequest-user-service /usr/local/bincodequest-user-service/
COPY --from=builder /workspace/target/release/codequest-user-service /usr/local/bin/codequest-user-service
ENTRYPOINT ["codequest-user-service"]
LABEL service=users

# codequest-quest-service
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
# COPY --from=builder /usr/local/cargo/bin/codequest-quest-service /usr/local/bincodequest-quest-service/
COPY --from=builder /workspace/target/release/codequest-quest-service /usr/local/bin/codequest-quest-service
ENTRYPOINT ["codequest-quest-service"]
LABEL service=quests

# codequest-progression-service
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
# COPY --from=builder /usr/local/cargo/bin/codequest-progression-service /usr/local/bincodequest-progression-service/
COPY --from=builder /workspace/target/release/codequest-progression-service /usr/local/bin/codequest-progression-service
ENTRYPOINT ["codequest-progression-service"]
LABEL service=progression
