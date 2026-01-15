```markdown
# Developing Sapphillon

# Developing Sapphillon

## Makefile targets

This section documents common Makefile targets and their purpose.

- `rust_test`: Run Rust tests for the entire workspace with all features enabled (`cargo test --workspace --all-features`).
- `rust_build`: Build the Rust project for the whole workspace with all features (`cargo build --workspace --all-features`).
- `rust_check_format`: Check Rust formatting and run clippy. The Makefile runs `cargo fmt --all --check || true` and `cargo clippy --workspace || true`, so these checks are non-fatal to the `make` invocation.
- `rust_fix_format`: Fix Rust formatting and attempt automatic clippy fixes (`cargo fmt --all || true` and `cargo clippy --workspace --fix --allow-dirty || true`).
- `gen_empty_db`: Create an empty SQLite database file at `./db/sqlite.db` (creates the `db` directory and touches the file).
- `migrate_generate`: Generate a SeaORM migration. Usage: `make migrate_generate NAME=your_migration_name` (this calls `sea-orm-cli migrate generate $(NAME)`).
- `migrate`: Run SeaORM migrations against `sqlite://db/sqlite.db` (creates `db/sqlite.db` if missing and runs `sea-orm-cli migrate up -u "sqlite://db/sqlite.db"`).
- `entity_generate`: Generate SeaORM entity code from the configured database into `./entity/src/entity`.
- `run`: Run the Rust application for local/debug use. This target creates `./debug/plugins` and starts the app with debug logging, using `./debug/sqlite.db` as the DB and saving external plugins to `./debug/plugins` (invokes `cargo run -- --loglevel debug --db-url ./debug/sqlite.db --ext-plugin-save-dir ./debug/plugins start`).
- `grpcui`: Launch `grpcui` against the local gRPC server (runs `grpcui -plaintext localhost:50051`).

If you need to run a sequence of tasks (for example create the DB, run migrations, and generate entities), run the targets in order:

`make gen_empty_db && make migrate && make entity_generate`

## Notes

- The formatting/check targets in the Makefile are tolerant: `rust_check_format` runs `cargo fmt --all --check || true` and `cargo clippy --workspace || true` so they won't cause `make` to fail. `rust_fix_format` runs `cargo fmt --all || true` and `cargo clippy --workspace --fix --allow-dirty || true` to attempt automatic fixes without aborting the make run.
- `gen_empty_db`, `migrate`, and `entity_generate` work with `./db/sqlite.db` (the Makefile creates `./db` and touches the file if missing). The `run` target uses a separate debug DB at `./debug/sqlite.db` and stores runtime plugin files under `./debug/plugins`.

## Permissions System

### Wildcard Permission

The permission system supports a wildcard `plugin_function_id` of `*`. When a workflow is granted a permission with this `plugin_function_id`, it is allowed to bypass all permission checks for all plugins. This is useful for testing and for workflows that are trusted to have full access to the system.