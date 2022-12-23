test-cli:
	cd modsurfer-cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/mod.yaml

empty-cli:
	cd modsurfer-cli
	cargo run -- validate -p test/spidermonkey.wasm -c test/empty.yaml
