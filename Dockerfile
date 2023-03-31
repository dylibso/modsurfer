FROM rust:latest as builder

RUN update-ca-certificates

WORKDIR /modsurfer
COPY . .

RUN cargo build --release

FROM ubuntu

RUN echo "deb http://security.ubuntu.com/ubuntu focal-security main" | tee /etc/apt/sources.list.d/focal-security.list
RUN apt-get update && apt-get -y install libssl1.1 bash

LABEL org.opencontainers.image.source=https://github.com/dylibso/modsurfer
LABEL org.opencontainers.image.description="Modsurfer CLI image, learn more at https://dev.dylib.so/docs/modsurfer/cli"
LABEL org.opencontainers.image.licenses=Apache-2.0

COPY --from=builder /modsurfer/target/release/modsurfer /usr/local/bin

ENTRYPOINT ["modsurfer"]