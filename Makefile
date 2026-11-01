CORE = core/Cargo.toml

.PHONY: wasm fixtures test

wasm:
	cargo build --release --manifest-path $(CORE) --target wasm32-unknown-unknown --features plugin
	mkdir -p src
	cp core/target/wasm32-unknown-unknown/release/florilegium_core.wasm src/florilegium.wasm

fixtures:
	tools/oracle/gen.sh

# tt has nothing to run until the Typst API exists; it is skipped when no tests/ directory is there.
test:
	cargo test --manifest-path $(CORE)
	@if [ -d tests ]; then tt run; fi
