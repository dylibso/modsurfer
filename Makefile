test-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/mod.yaml

empty-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/empty.yaml

unknown-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/unknown-fields.yaml

install:
	@echo "if this fails, be sure to install protoc via: https://grpc.io/docs/protoc-installation/"
	@echo "check PROTOC: `which protoc`"
	go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.32.0
	cargo install protobuf-codegen@3.4.0

generate: install
	protoc --experimental_allow_proto3_optional \
		--rust_out=proto/v1/src \
		--go_out=proto/v1/go \
		--php_out=proto/v1/php \
		proto/v1/api.proto

generate-web-list-mods-response:
	cd scripts/protobuf-list-modules-response && cargo run --release -- $(take)
	mv scripts/protobuf-list-modules-response/ListModulesResponse.pb api/
