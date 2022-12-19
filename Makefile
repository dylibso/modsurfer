test-cli:
	cargo run --bin modsurfer -- validate -p test/spidermonkey.wasm -c test/mod.yaml

empty-cli:
	cargo run --bin modsurfer -- validate -p test/spidermonkey.wasm -c test/empty.yaml
