.PHONY: all install test lint kani mutants parity examples contracts clean

all: lint test

install:
	cargo install kani-verifier
	cargo kani setup
	cargo install cargo-mutants

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

test:
	cargo test --lib
	cargo test --test golden_vectors

kani:
# Every harness, one at a time (-Z stubbing: some replace sort, sqrt, sin_cos or cpuid; see src/kani_harnesses.rs).
	cargo kani -Z stubbing -j 1

mutants:
	cargo mutants -j4 -- --lib

parity:
	uv run tests/falsify_parity.py

examples:
	@for ex in sort stats matrix solve optimize integrate fourier random monte_carlo graph parity; do \
		echo "=== $$ex ==="; \
		cargo run --example $$ex --quiet; \
	done

contracts:
	@for f in contracts/*.yaml; do \
		[ "$$(basename $$f)" = "binding.yaml" ] && continue; \
		pv validate "$$f"; \
	done
	pv score contracts/ --binding contracts/binding.yaml

clean:
	cargo clean
	rm -rf mutants.out
