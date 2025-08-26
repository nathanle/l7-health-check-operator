FROM rust:1.46.0-alpine AS builder
# build project
RUN apk add musl-dev
WORKDIR /usr/src/
RUN USER=root cargo new health-check-operator
WORKDIR /usr/src/health-check-operator
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

# Bundle Stage
FROM alpine as final
COPY --from=builder /usr/src/health-check-operator/target/release/health-check-operator .
CMD ["./health-check-operator"]
