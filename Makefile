test-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/mod.yaml

empty-cli:
	cd cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/empty.yaml
