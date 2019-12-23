target/release/regular: src/main.rs src/lib.rs src/cangwu.rs src/te.rs
	cargo build --release

target/debug/regular: src/main.rs src/lib.rs src/cangwu.rs src/te.rs
	cargo build

pkg/regular_bg.wasm: src/wasm.rs src/lib.rs src/cangwu.rs src/te.rs
	wasm-pack build --target web

regular_bg.wasm: pkg/regular_bg.wasm
	wasm-opt -O4 pkg/regular_bg.wasm -o regular_bg.wasm

.PHONY: doc
doc:
	cargo doc --target wasm32-unknown-unknown

.PHONY: test
test:
	cargo test

.PHONY: wasm
wasm: pkg/regular_bg.wasm

.PHONY: wasm-release
wasm-release: regular_bg.wasm

.PHONY: debug
debug: target/debug/regular
