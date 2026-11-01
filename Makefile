CORE = core/Cargo.toml

.PHONY: wasm fixtures test

wasm:
	cargo build --manifest-path $(CORE) --target wasm32-unknown-unknown --profile wasm --features plugin
	mkdir -p src
	cp core/target/wasm32-unknown-unknown/wasm/florilegium_core.wasm src/florilegium.wasm

fixtures:
	tools/oracle/gen.sh

test:
	cargo test --manifest-path $(CORE)
	tt run
