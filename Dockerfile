# Start with a rust alpine image
FROM rust:1-alpine3.21 AS builder
# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"
# if needed, add additional dependencies here
RUN apk add --no-cache musl-dev
# set the workdir and copy the source into it
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY src /app/src

# do a release build
RUN cargo build --release
RUN cargo test --release
RUN strip target/release/spacecheck

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.21

LABEL org.opencontainers.image.source=https://github.com/agrenott/spacecheck
LABEL org.opencontainers.image.description="spacecheck Docker image"
LABEL org.opencontainers.image.licenses=MIT

# if needed, install additional dependencies here
RUN apk add --no-cache libgcc
# copy the binary into the final image
COPY --from=builder /app/target/release/spacecheck .

# Exposed volume and port
VOLUME /monitored_fs
EXPOSE 8080/tcp

# set the binary as entrypoint
ENTRYPOINT ["/spacecheck"]
CMD ["/monitored_fs"]
