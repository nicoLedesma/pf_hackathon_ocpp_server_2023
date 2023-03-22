####################################################################################################
### STAGE 1: Build Container: build crate and dependencies
####################################################################################################
FROM lukemathwalker/cargo-chef:latest-rust-1.68 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Cache dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

####################################################################################################
### STAGE 2: Execution Container: copy the static binary from build container and run
####################################################################################################
# Read regarding this base image's security flaws:
# https://www.redhat.com/en/blog/why-distroless-containers-arent-security-solution-you-think-they-are
FROM gcr.io/distroless/cc:nonroot

# Copy binary and set entrypoint.
COPY --from=builder /app/target/release/ocpp_server ocpp_server
CMD ["./ocpp_server"]
