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
	cargo kani --harness verify_quicksort_sorted
	cargo kani --harness verify_quicksort_preserves_length
	cargo kani --harness verify_variance_non_negative
	cargo kani --harness verify_mean_constant
	cargo kani --harness verify_correlation_bounded

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
