.PHONY: rust_test, rust_build, rust_check_format, rust_fix_format 

rust_test:
	@echo "Run Rust Tests"
	@echo "----------------------------------------------------------"
	cargo test --workspace --all-features
	@echo "----------------------------------------------------------"

rust_build:
	@echo "Build Rust Project"
	@echo "----------------------------------------------------------"
	cargo build --workspace --all-features
	@echo "----------------------------------------------------------"

rust_check_format:
	@echo "Check Rust Format"
	@echo "----------------------------------------------------------"
	cargo fmt --all --check || true
	@echo "----------------------------------------------------------"
	cargo clippy --workspace || true
	@echo "----------------------------------------------------------"

rust_fix_format:
	@echo "Fix Rust Format"
	@echo "----------------------------------------------------------"
	cargo fmt --all || true
	@echo "----------------------------------------------------------"
	cargo clippy --workspace --fix --allow-dirty || true
	@echo "----------------------------------------------------------"

