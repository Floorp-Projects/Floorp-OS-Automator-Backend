// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

pub mod ext_plugin;
pub mod model;
pub mod permission;
pub mod plugin;
pub mod provider;
pub mod workflow;

#[cfg(test)]
use sea_orm::{Database, DatabaseConnection, DbErr};

/// Simple helper for database crate tests to obtain an in-memory SQLite connection.
#[cfg(test)]
pub struct TestState {
    db_url: &'static str,
}

#[cfg(test)]
impl TestState {
    pub const fn new_in_memory() -> Self {
        Self {
            db_url: "sqlite::memory:?cache=shared",
        }
    }

    pub async fn get_db_connection(&self) -> Result<DatabaseConnection, DbErr> {
        Database::connect(self.db_url).await
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! global_state_for_tests {
    () => {{ $crate::TestState::new_in_memory() }};
}
