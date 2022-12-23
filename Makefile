test-cli:
	cd modsurfer-cli
	cargo run --features modsurfer/reqwest -- validate -p test/spidermonkey.wasm -c test/mod.yaml

empty-cli:
	cd modsurfer-cli
	cargo run --bin modsurfer -- validate -p test/spidermonkey.wasm -c test/empty.yaml
