FROM clux/muslrust:stable AS builder

COPY Cargo.* .
COPY src/ src/

RUN --mount=type=cache,target=/volume/target \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --bin cs523 && \
    mv /volume/target/x86_64-unknown-linux-musl/release/cs523 .

FROM cgr.dev/chainguard/busybox:latest
COPY --from=builder --chown=nonroot:nonroot /volume/cs523 /app/cs523
EXPOSE 8443
ENTRYPOINT ["/app/cs523"]