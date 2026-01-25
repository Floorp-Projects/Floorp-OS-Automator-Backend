.PHONY: test, build, fmt, fix_fmt, gen_empty_db, migrate_generate, entity_generate, run, grpcui, migrate, help

help:
	@echo "Available make targets:"
	@echo
	@echo "  test             Run Rust tests"
	@echo "  build            Build Rust project"
	@echo "  fmt              Check Rust formatting and run clippy (non-failing)"
	@echo "  fix_fmt          Fix formatting and apply clippy fixes"
	@echo "  release          Build in release mode"
	@echo "  full_test        Run build, fmt, test and release"
	@echo "  gen_empty_db     Create empty sqlite DB at ./db/sqlite.db"
	@echo "  migrate_generate Generate a SeaORM migration (NAME=... required)"
	@echo "  migrate          Run SeaORM migrations against sqlite://db/sqlite.db"
	@echo "  entity_generate  Generate SeaORM entities from database"
	@echo "  run              Run the Rust application (debug mode)"
	@echo "  grpcui           Run grpcui against localhost:50051"
	@echo
	@echo "Usage: make <target> [VARIABLE=value]"

test:
	@echo "Run Rust Tests"
	@echo "----------------------------------------------------------"
	cargo build --workspace --all-features
	RUST_TEST_THREADS=1 cargo test --workspace --all-features
	@echo "----------------------------------------------------------"

build:
	@echo "Build Rust Project"
	@echo "----------------------------------------------------------"
	cargo build --workspace --all-features
	@echo "----------------------------------------------------------"

fmt:
	@echo "Check Rust Format"
	@echo "----------------------------------------------------------"
	cargo fmt --all --check || true
	@echo "----------------------------------------------------------"
	cargo clippy --workspace || true
	@echo "----------------------------------------------------------"

fix_fmt:
	@echo "Fix Rust Format"
	@echo "----------------------------------------------------------"
	cargo fmt --all || true
	@echo "----------------------------------------------------------"
	cargo clippy --workspace --fix --allow-dirty || true
	@echo "----------------------------------------------------------"

release:
	@echo "Build Rust Project in Release Mode"
	@echo "----------------------------------------------------------"
	cargo build --workspace --all-features --release
	@echo "----------------------------------------------------------"

full_test: build fmt test release
	@echo "Full Rust Test Completed"

gen_empty_db:
	@echo "Generate empty SQLite database"
	@echo "----------------------------------------------------------"
	mkdir -p ./db
	touch ./db/sqlite.db
	@echo "----------------------------------------------------------"

migrate_generate:
	@echo "Generate SeaORM migration"
	@echo "----------------------------------------------------------"
	@if [ -z "$(NAME)" ]; then \
		echo "Usage: make migrate_generate NAME=your_migration_name"; exit 1; \
	fi
	sea-orm-cli migrate generate $(NAME)
	@echo "----------------------------------------------------------"

migrate:
	@echo "Run SeaORM migrations against sqlite://db/sqlite.db"
	@echo "----------------------------------------------------------"
	mkdir -p ./db
	touch ./db/sqlite.db
	sea-orm-cli migrate up -u "sqlite://db/sqlite.db"
	@echo "----------------------------------------------------------"

entity_generate:
	@echo "Generate SeaORM entities from database"
	@echo "----------------------------------------------------------"
	mkdir -p ./db
	touch ./db/sqlite.db
	sea-orm-cli generate entity -u "sqlite://db/sqlite.db" -o ./entity/src/entity
	@echo "----------------------------------------------------------"

run:
	@echo "Run the Rust Application"
	@echo "auto make dubug folder and put system data."
	@echo "----------------------------------------------------------"
	mkdir -p ./debug/plugins
	cargo run -- --loglevel debug --db-url "sqlite://./debug/sqlite.db" --ext-plugin-save-dir ./debug/plugins start
	@echo "----------------------------------------------------------"

grpcui:
	@echo "Run gRPC UI for the Rust Application"
	@echo "----------------------------------------------------------"
	grpcui -plaintext localhost:50051
	@echo "----------------------------------------------------------"
