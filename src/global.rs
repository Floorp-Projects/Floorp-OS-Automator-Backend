// Sapphillon
// Copyright 2025 Yuta Takahashi
//
// This file is part of Sapphillon
//
// Sapphillon is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use sea_orm::{Database, DatabaseConnection};
use std::sync::LazyLock;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct GlobalStateData {
    db_initialized: bool,
    db_url: String,
}

#[derive(Debug)]
pub struct GlobalState {
    pub data: LazyLock<RwLock<GlobalStateData>>,
}

impl GlobalState {
    pub const fn new() -> Self {
        GlobalState {
            data: LazyLock::new(|| {
                RwLock::new(GlobalStateData {
                    db_initialized: false,
                    db_url: String::new(),
                })
            }),
        }
    }

    pub async fn get_db_connection(&self) -> anyhow::Result<DatabaseConnection> {
        let data = self.data.read().await;
        if data.db_url.is_empty() {
            anyhow::bail!("Database URL is not set");
        }
        let conn = Database::connect(&data.db_url).await?;
        Ok(conn)
    }

    pub async fn wait_init_and_get_connection(&self) -> anyhow::Result<DatabaseConnection> {
        self.wait_db_initialized().await?;
        self.get_db_connection().await
    }

    pub async fn async_set_db_url(&self, url: String) {
        let mut data = self.data.write().await;
        data.db_url = url;
    }

    pub fn set_db_url(self: std::sync::Arc<Self>, url: String) {
        tokio::spawn(async move {
            let mut data = self.data.write().await;
            data.db_url = url;
        });
    }

    pub async fn async_get_db_url(&self) -> String {
        let data = self.data.read().await;
        data.db_url.clone()
    }

    pub fn get_db_url_blocking(self: std::sync::Arc<Self>) -> String {
        // Use block_in_place to synchronously block on the async read
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let data = self.data.read().await;
                data.db_url.clone()
            })
        })
    }

    pub async fn async_set_db_initialized(&self, initialized: bool) {
        let mut data = self.data.write().await;
        data.db_initialized = initialized;
    }

    pub fn set_db_initialized(self: std::sync::Arc<Self>, initialized: bool) {
        tokio::spawn(async move {
            self.async_set_db_initialized(initialized).await;
        });
    }

    pub fn is_db_initialized(&self) -> bool {
        // Use the non-blocking try_read so we don't block the current thread
        // (blocking_read would panic if called from a Tokio runtime thread).
        match self.data.try_read() {
            Ok(data) => data.db_initialized,
            Err(_) => {
                // If we can't acquire the lock, assume not initialized
                false
            }
        }
    }

    pub async fn wait_db_initialized(&self) -> anyhow::Result<()> {
        let mut count = 1;
        loop {
            {
                let data = self.data.read().await;
                if data.db_initialized {
                    break;
                }
            }
            // Sleep briefly to avoid busy waiting
            log::info!("Waiting for DB to be initialized...{count}");
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            count += 1;

            if (count > 10) {
                log::warn!("Still waiting for DB to be initialized after {count} seconds...");
            }
            if (count > 60) {
                log::error!("Waited over a minute for DB to be initialized, giving up.");
                anyhow::bail!("Timeout waiting for DB to be initialized");
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for GlobalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the non-blocking try_read so we don't block the current thread
        // (blocking_read would panic if called from a Tokio runtime thread).
        match self.data.try_read() {
            Ok(data) => write!(
                f,
                "GlobalState {{ db_initialized: {}, db_url: '{}' }}",
                data.db_initialized, data.db_url
            ),
            Err(_) => write!(f, "GlobalState {{ <locked> }}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn display_unlocked_shows_defaults() {
        let gs = GlobalState::new();
        let s = format!("{gs}");
        assert!(
            s.contains("db_initialized: false"),
            "display should show db_initialized: false, got: {s}"
        );
        assert!(
            s.contains("db_url: ''"),
            "display should show empty db_url, got: {s}"
        );
    }

    #[test]
    fn display_locked_shows_locked() {
        let gs = GlobalState::new();

        // Acquire an exclusive write lock so try_read inside Display fails
        let write_guard = gs.data.try_write().expect("should acquire write lock");
        let s = format!("{gs}");
        assert!(
            s.contains("<locked>"),
            "display should indicate locked state, got: {s}"
        );

        // drop the guard to release lock (explicit for clarity)
        drop(write_guard);
    }

    #[test]
    fn mutate_state_and_display() {
        let gs = GlobalState::new();

        {
            // Acquire write lock and change fields
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_initialized = true;
            w.db_url = "sqlite://:memory:".to_string();
        }

        let s = format!("{gs}");
        assert!(
            s.contains("db_initialized: true"),
            "display should show db_initialized: true, got: {s}"
        );
        assert!(
            s.contains("sqlite://:memory:"),
            "display should show updated db_url, got: {s}"
        );
    }

    #[test]
    fn is_db_initialized_reflects_state_changes_and_locked() {
        let gs = GlobalState::new();

        // initially false
        assert!(!gs.is_db_initialized());

        // set to true and release lock
        {
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_initialized = true;
        }

        // now should report true
        assert!(
            gs.is_db_initialized(),
            "is_db_initialized should return true after setting the flag"
        );

        // if we hold the write lock, try_read inside is_db_initialized will fail -> returns false
        let mut guard = gs.data.try_write().expect("should acquire write lock");
        guard.db_initialized = true;
        assert!(
            !gs.is_db_initialized(),
            "is_db_initialized should return false while locked (try_read fails)"
        );
        drop(guard);
    }

    #[tokio::test]
    async fn wait_db_initialized_returns_if_already_initialized() {
        let gs = GlobalState::new();

        // set to initialized
        {
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_initialized = true;
        }

        // should return immediately (use a timeout to protect the test)
        let res = tokio::time::timeout(Duration::from_secs(1), gs.wait_db_initialized()).await;
        assert!(
            res.is_ok(),
            "wait_db_initialized should return immediately when already initialized"
        );
    }

    #[tokio::test]
    async fn wait_db_initialized_waits_until_flag_set() {
        let gs = Arc::new(GlobalState::new());

        // spawn a task that sets the flag after a short delay
        let gs_clone = Arc::clone(&gs);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let mut w = gs_clone.data.write().await;
            w.db_initialized = true;
        });

        // wait_db_initialized should complete once the flag is set; use timeout so test fails fast on regressions
        let res = tokio::time::timeout(Duration::from_secs(2), gs.wait_db_initialized()).await;
        assert!(
            res.is_ok(),
            "wait_db_initialized should return after the flag is set by another task"
        );
    }

    #[tokio::test]
    async fn concurrent_waiters_unblocked() {
        let gs = Arc::new(GlobalState::new());

        // Spawn multiple waiters that should all unblock once the flag is set
        let mut handles = Vec::new();
        for _ in 0..5 {
            let gs_clone = Arc::clone(&gs);
            handles.push(tokio::spawn(async move {
                // each waiter will time out after 2 seconds to keep the test fast
                tokio::time::timeout(Duration::from_secs(2), gs_clone.wait_db_initialized()).await
            }));
        }

        // set the flag shortly after
        let gs_set = Arc::clone(&gs);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let mut w = gs_set.data.write().await;
            w.db_initialized = true;
        });

        // ensure all waiters completed successfully
        for h in handles {
            let res = h.await.expect("waiter task panicked");
            assert!(res.is_ok(), "waiter timed out before db was initialized");
            // inner result is anyhow::Result<()> returned by wait_db_initialized
            res.unwrap().expect("wait_db_initialized returned an error");
        }
    }

    #[test]
    fn db_url_mutation_and_display_shows_value() {
        let gs = GlobalState::new();

        {
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_url = "postgres://localhost/mydb".to_string();
        }

        let s = format!("{gs}");
        assert!(
            s.contains("postgres://localhost/mydb"),
            "display should show updated db_url, got: {s}"
        );
    }

    #[tokio::test]
    async fn async_set_db_initialized_sets_flag_immediately() {
        let gs = GlobalState::new();

        // call the async setter and await it
        gs.async_set_db_initialized(true).await;

        // try_read should succeed and report true
        assert!(
            gs.is_db_initialized(),
            "async_set_db_initialized should set the flag to true"
        );
    }

    #[tokio::test]
    async fn set_db_initialized_spawns_and_sets_flag() {
        use std::sync::Arc;
        let gs = Arc::new(GlobalState::new());

        // Call the non-async setter which spawns a background task
        let gs_clone = Arc::clone(&gs);
        gs_clone.set_db_initialized(true);

        // Wait up to 1 second for the spawned task to complete and set the flag
        let res = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            // poll until flag becomes true
            for _ in 0..20 {
                if gs.is_db_initialized() {
                    return true;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            false
        })
        .await
        .expect("timeout waiting for spawn to set flag");

        assert!(res, "set_db_initialized spawn did not set the flag in time");
    }

    #[tokio::test]
    async fn async_set_and_get_db_url_roundtrip() {
        let gs = GlobalState::new();

        // set the url asynchronously and then read it back
        gs.async_set_db_url("sqlite://async-test".to_string()).await;
        let got = gs.async_get_db_url().await;
        assert_eq!(got, "sqlite://async-test");
    }

    #[test]
    fn get_db_url_blocking_returns_value_from_background() {
        use std::sync::Arc;

        let gs = Arc::new(GlobalState::new());

        // set value via direct write so the blocking reader can observe it
        {
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_url = "postgres://blocking".to_string();
        }

        // Run the blocking getter inside a Tokio runtime so block_in_place and
        // Handle::current() succeed. We run it on a separate thread that
        // creates its own runtime to avoid interfering with the test harness.
        let gs_clone = Arc::clone(&gs);
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("create runtime");
            rt.block_on(async move { gs_clone.get_db_url_blocking() })
        });
        let got = handle.join().expect("thread panicked");
        assert_eq!(got, "postgres://blocking");
    }

    #[tokio::test]
    async fn set_db_url_spawns_and_sets_value() {
        use std::sync::Arc;
        let gs = Arc::new(GlobalState::new());

        // call the non-async setter which spawns a background task
        let gs_clone = Arc::clone(&gs);
        gs_clone.set_db_url("mysql://spawned".to_string());

        // Wait up to 1 second for the spawned task to complete and set the url
        let res = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            for _ in 0..20 {
                let cur = gs.async_get_db_url().await;
                if cur == "mysql://spawned" {
                    return true;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            false
        })
        .await
        .expect("timeout waiting for spawn to set db_url");

        assert!(res, "set_db_url spawn did not set the db_url in time");
    }

    #[tokio::test]
    async fn get_db_connection_returns_connection_when_url_set() {
        // Ensure that when a valid DB URL is present, get_db_connection returns a connection
        let gs = GlobalState::new();

        // set an in-memory sqlite URL
        {
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_url = "sqlite::memory:".to_string();
        }

        let conn = gs.get_db_connection().await;
        assert!(
            conn.is_ok(),
            "get_db_connection should return Ok when db_url is set"
        );
    }

    #[tokio::test]
    async fn wait_init_and_get_connection_waits_and_returns_connection() {
        use std::sync::Arc;

        // Start with an uninitialized state; a background task will set the URL and initialized flag.
        let gs = Arc::new(GlobalState::new());

        let gs_clone = Arc::clone(&gs);
        tokio::spawn(async move {
            // give the waiter a moment to start waiting
            tokio::time::sleep(Duration::from_millis(200)).await;
            let mut w = gs_clone.data.write().await;
            w.db_url = "sqlite::memory:".to_string();
            w.db_initialized = true;
        });

        // wait_init_and_get_connection should wait until the flag is set and then return a connection
        let res =
            tokio::time::timeout(Duration::from_secs(3), gs.wait_init_and_get_connection()).await;
        assert!(
            res.is_ok(),
            "wait_init_and_get_connection should complete within timeout"
        );
        let conn = res.unwrap();
        assert!(
            conn.is_ok(),
            "wait_init_and_get_connection should return a valid connection"
        );
    }
}
