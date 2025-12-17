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
RUN cargo install --path quest-service

# codequest-gateway
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /workspace/Rocket.toml /app/
COPY --from=builder /workspace/target/release/codequest-quest-service /usr/local/bin/codequest-quest-service
# RUN chown -R nonroot:nonroot /app
# USER nonroot:nonroot
ENTRYPOINT ["codequest-quest-service"]
