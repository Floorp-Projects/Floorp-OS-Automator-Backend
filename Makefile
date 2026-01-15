.PHONY: rust_test, rust_build, rust_check_format, rust_fix_format, gen_empty_db, migrate_generate, entity_generate, run, grpcui, migrate

rust_test:
	@echo "Run Rust Tests"
	@echo "----------------------------------------------------------"
	RUST_TEST_THREADS=1 cargo test --workspace --all-features
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
