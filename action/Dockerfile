LABEL org.opencontainers.image.source=https://github.com/dylibso/modsurfer
LABEL org.opencontainers.image.description="Modsurfer CLI image, learn more at https://dylib.so"
LABEL org.opencontainers.image.licenses=Apache-2.0

FROM rust:latest as builder

RUN update-ca-certificates

WORKDIR /modsurfer
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /cli

COPY --from=builder /modsurfer/target/release/modsurfer ./

ENTRYPOINT ["/cli/modsurfer"]