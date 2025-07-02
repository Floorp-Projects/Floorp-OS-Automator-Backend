.PHONY: buf_check_style, buf_fix_style, buf_debug, rust_test, rust_build, rust_check_format, rust_fix_format

buf_check_style:
	@echo "Check Protocol Buffer Style"
	@echo "----------------------------------------------------------"
	buf lint proto || true
	@echo "----------------------------------------------------------"
	buf format proto -d || true

buf_fix_style:
	@echo "Fix Protocol Buffer Style"
	@echo "----------------------------------------------------------"
	buf format proto -w
	@echo "----------------------------------------------------------"

buf_debug:
	@echo "Debug Protocol Buffer"
	@proto_files=$$(find proto -name '*.proto' | tr '\n' ' '); \
	if [ -z "$$proto_files" ]; then \
		echo "No .proto files found in proto/"; \
		exit 1; \
	fi; \
	evans --proto $$proto_files repl

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

