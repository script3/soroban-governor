default: build

test: build
	cargo test --all --tests

build:
	mkdir -p target/wasm32-unknown-unknown/optimized

	cargo rustc --manifest-path=contracts/votes/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features soroban-votes/sep-0041
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/soroban_votes.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/soroban_votes.wasm

	cargo rustc --manifest-path=contracts/votes/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features soroban-votes/bonding
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/soroban_votes.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/soroban_votes_bonding.wasm

	cargo rustc --manifest-path=contracts/governor/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/soroban_governor.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/soroban_governor.wasm

	cargo rustc --manifest-path=contracts/mock-subcall/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release

	cd target/wasm32-unknown-unknown/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

fmt:
	cargo fmt --all

clean:
	cargo clean

generate-js:
	soroban contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32-unknown-unknown/optimized/soroban_votes_bonding.wasm --output-dir ./js/js-votes-bonding/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone
	soroban contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32-unknown-unknown/optimized/soroban_votes.wasm --output-dir ./js/js-votes/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone
	soroban contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32-unknown-unknown/optimized/soroban_governor.wasm --output-dir ./js/js-governor/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone
