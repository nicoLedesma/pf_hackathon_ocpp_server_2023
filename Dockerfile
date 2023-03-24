# TODO add kubernetes resource limits to this container
# TODO run image scanner on this image in our CI/CD

####################################################################################################
### STAGE 1: Build Container: build crate and dependencies
####################################################################################################
FROM rust:1.68-alpine as builder
WORKDIR /usr/src/app

# For compressing executables
RUN apk add upx

# Build application
COPY . .
RUN cargo build --release
RUN upx /app/target/release/ocpp_server

####################################################################################################
### STAGE 2: Execution Container: copy the static binary from build container and run
####################################################################################################
# Read regarding this base image's security flaws:
# https://www.redhat.com/en/blog/why-distroless-containers-arent-security-solution-you-think-they-are
FROM gcr.io/distroless/cc:nonroot

# Default value is written here for clarity.
WORKDIR /home/nonroot

# Copy binary and set entrypoint.
COPY --from=builder /app/target/release/ocpp_server ocpp_server
CMD ["./ocpp_server"]
