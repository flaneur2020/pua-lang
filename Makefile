.PHONY: setup
setup:
	cargo install wasm-gc
	(cd web && yarn)

.PHONY: start
start:
	make build_wasm
	(cd web && yarn start)

.PHONY: test
test:
	cargo test

.PHONY: build_repl
build_repl:
	cargo build --release

.PHONY: build_wasm
build_wasm:
	cargo build --bin wasm --release --target wasm32-unknown-unknown --features=wasm
	wasm-gc target/wasm32-unknown-unknown/release/wasm.wasm web/src/monkey.wasm

.PHONY: web_deploy
web_deploy:
	make build_wasm
	(cd web && yarn --pure-lockfile && yarn deploy)

.PHONY: repl
repl:
	cargo run --bin pua-lang --features="binaries"
