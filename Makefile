.PHONY: rust_test, rust_build, rust_check_format, rust_fix_format 

rust_test:
	@echo "Run Rust Tests"
	@echo "----------------------------------------------------------"
	cargo test
	@echo "----------------------------------------------------------"

rust_build:
	@echo "Build Rust Project"
	@echo "----------------------------------------------------------"
	cargo build
	@echo "----------------------------------------------------------"

rust_check_format:
	@echo "Check Rust Format"
	@echo "----------------------------------------------------------"
	cargo fmt --check
	cargo clippy
	@echo "----------------------------------------------------------"

rust_fix_format:
	@echo "Fix Rust Format"
	@echo "----------------------------------------------------------"
	cargo fmt
	cargo clippy --fix --allow-dirty
	@echo "----------------------------------------------------------"

