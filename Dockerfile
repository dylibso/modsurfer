FROM rust:latest as builder

RUN update-ca-certificates

WORKDIR /modsurfer
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc

LABEL org.opencontainers.image.source=https://github.com/dylibso/modsurfer
LABEL org.opencontainers.image.description="Modsurfer CLI image, learn more at https://dev.dylib.so/docs/modsurfer/cli"
LABEL org.opencontainers.image.licenses=Apache-2.0

WORKDIR /cli

COPY --from=builder /modsurfer/target/release/modsurfer ./

ENTRYPOINT ["/cli/modsurfer"]