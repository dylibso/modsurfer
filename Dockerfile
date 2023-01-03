FROM rust:latest

WORKDIR /modsurfer
COPY . .

RUN cargo install --path cli

ENTRYPOINT ["modsurfer", "validate", "-p", "test/spidermonkey.wasm", "-c", "test/mod.yaml"]