test-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/mod.yaml

empty-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/empty.yaml

install:
	@echo "check PROTOC: `which protoc`"
	cargo install protobuf-codegen

generate: install
	protoc --experimental_allow_proto3_optional --rust_out=proto/v1/src proto/v1/api.proto

