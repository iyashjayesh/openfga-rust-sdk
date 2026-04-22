.PHONY: fmt lint test check security build doc

# Format source code
fmt:
	cargo fmt --all

# Lint (format check + clippy)
lint: fmt
	cargo clippy --all-features -- -D warnings

# Run all tests
test:
	cargo test --all-features

# Run security audit
security:
	cargo audit
	cargo deny check

# Run everything (matches Go SDK's `make check`)
check: lint test security

# Build (debug)
build:
	cargo build --all-features

# Build documentation
doc:
	cargo doc --all-features --no-deps --open

# Format + check + test (quick loop)
ci: fmt lint test
