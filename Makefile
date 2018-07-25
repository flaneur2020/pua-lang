.PHONY: setup
setup:
	yarn
	cargo install wasm-gc

.PHONY: start
start:
	yarn start

.PHONY: test
test:
	cargo test

.PHONY: build_repl
build_repl:
	cargo build --release

.PHONY: build_wasm
build_wasm:
	cargo build --bin wasm --release --target wasm32-unknown-unknown
	wasm-gc target/wasm32-unknown-unknown/release/wasm.wasm web/src/monkey.wasm

.PHONY: web_deploy
web_deploy:
	(cd web && yarn --pure-lockfile && yarn deploy)

.PHONY: repl
repl:
	cargo run --bin monkey --features="binaries"
