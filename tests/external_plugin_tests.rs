// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! External plugin integration tests.
//!
//! This test suite verifies the complete flow of external plugin installation,
//! loading, and execution within workflows.
//!
//! ## Test Organization
//!
//! Tests are organized into the following modules:
//!
//! - **installation**: Plugin installation and filesystem operations (no external dependencies)
//! - **bridge**: rsjs_bridge_core function execution (requires external plugin server)
//! - **workflow**: Workflow execution with external plugins (requires external plugin server)
//! - **e2e**: End-to-end install→load→execute flow (requires external plugin server)
//!
//! ## Running Tests
//!
//! Run tests that don't require external infrastructure:
//! ```bash
//! cargo test --test external_plugin
//! ```
//!
//! Run tests that require the external plugin server:
//! ```bash
//! # First, build the external plugin server
//! cargo build --release -p ext_plugin
//!
//! # Then run ignored tests
//! cargo test --test external_plugin -- --ignored
//! ```

mod external_plugin;
