FROM rust:latest as builder

RUN update-ca-certificates

WORKDIR /modsurfer
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /cli

COPY --from=builder /modsurfer/target/release/modsurfer ./

ENTRYPOINT ["/cli/modsurfer"]