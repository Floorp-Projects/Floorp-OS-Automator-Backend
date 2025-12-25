// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use sea_orm::{Database, DatabaseConnection, DbErr};

/// Helper used by unit tests to open an in-memory SQLite connection.
#[allow(dead_code)]
pub struct TestState {
    db_url: &'static str,
}

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

#[macro_export]
macro_rules! global_state_for_tests {
    () => {{ $crate::test_support::TestState::new_in_memory() }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_memory_connection() {
        let state = TestState::new_in_memory();
        let conn = state.get_db_connection().await;
        assert!(conn.is_ok());
    }
}
