# Developing Sapphillon

## Makefile targets

This section documents the project's common Makefile targets and their purpose.

- `rust_test`: Run Rust tests for the entire workspace with all features enabled (`cargo test --workspace --all-features`).
- `rust_build`: Build the Rust project for the whole workspace with all features (`cargo build --workspace --all-features`).
- `rust_check_format`: Check Rust formatting and run clippy without failing the make process (runs `cargo fmt --all --check` and `cargo clippy`).
- `rust_fix_format`: Fix Rust formatting and attempt automatic clippy fixes (`cargo fmt --all` and `cargo clippy --workspace --fix --allow-dirty`).
- `gen_empty_db`: Create an empty SQLite database file at `./db/sqlite.db` (creates the `db` directory and touches the file).
- `migrate_generate`: Generate a SeaORM migration. Usage: `make migrate_generate NAME=your_migration_name` (this calls `cargo run -p migration -- migrate generate $(NAME)`).
- `migrate`: Run SeaORM migrations against `sqlite://db/sqlite.db` (creates `db/sqlite.db` if missing and runs `sea-orm-cli migrate up -u "sqlite://db/sqlite.db"`).
- `entity_generate`: Generate SeaORM entity code from the configured database into `./entity/src/entity`.

If you need to run a sequence of tasks (for example create the DB, run migrations, and generate entities), run the targets in order: `make gen_empty_db && make migrate && make entity_generate`.