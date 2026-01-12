// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use sea_orm::{Database, DatabaseConnection};
use std::sync::LazyLock;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct GlobalStateData {
    db_initialized: bool,
    db_url: String,
    ext_plugin_save_dir: Option<String>,
}

#[derive(Debug)]
pub struct GlobalState {
    pub data: LazyLock<RwLock<GlobalStateData>>,
}

impl GlobalState {
    /// Creates a new [`GlobalState`] with default, uninitialized database settings.
    ///
    /// # Arguments
    ///
    /// This constructor takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`GlobalState`] instance backed by a lazily initialized [`RwLock`].
    pub const fn new() -> Self {
        GlobalState {
            data: LazyLock::new(|| {
                RwLock::new(GlobalStateData {
                    db_initialized: false,
                    db_url: String::new(),
                    ext_plugin_save_dir: None,
                })
            }),
        }
    }

    /// Opens a new database connection using the recorded URL after ensuring initialization.
    ///
    /// # Arguments
    ///
    /// This asynchronous method takes no additional arguments beyond the borrowed [`GlobalState`].
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] when the URL is present and initialization is complete, or an error if the state is invalid.
    pub async fn get_db_connection(&self) -> anyhow::Result<DatabaseConnection> {
        let data = self.data.read().await;

        if !data.db_initialized {
            anyhow::bail!("Database is not initialized");
        }

        if data.db_url.is_empty() {
            anyhow::bail!("Database URL is not set");
        }
        let conn = Database::connect(&data.db_url).await?;
        Ok(conn)
    }

    /// Waits for database initialization to complete and then provides a connection handle.
    ///
    /// # Arguments
    ///
    /// This asynchronous method takes no additional arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] once initialization succeeds, or an error if waiting fails.
    pub async fn wait_init_and_get_connection(&self) -> anyhow::Result<DatabaseConnection> {
        self.wait_db_initialized().await?;
        self.get_db_connection().await
    }

    /// Stores the database URL asynchronously, replacing any previous value.
    ///
    /// # Arguments
    ///
    /// * `url` - The connection string to persist for future database operations.
    ///
    /// # Returns
    ///
    /// Returns `()` once the URL has been written to the shared state.
    pub async fn async_set_db_url(&self, url: String) {
        let mut data = self.data.write().await;
        data.db_url = url;
    }

    /// Spawns a background task that updates the stored database URL.
    ///
    /// # Arguments
    ///
    /// * `self` - An [`Arc`] clone of the global state used inside the spawned task.
    /// * `url` - The database connection string to record.
    ///
    /// # Returns
    ///
    /// Returns immediately after scheduling the write operation in a background task.
    pub fn set_db_url(self: std::sync::Arc<Self>, url: String) {
        tokio::spawn(async move {
            let mut data = self.data.write().await;
            data.db_url = url;
        });
    }

    /// Reads the stored database URL asynchronously.
    ///
    /// # Arguments
    ///
    /// This method takes no additional arguments beyond the borrowed [`GlobalState`].
    ///
    /// # Returns
    ///
    /// Returns a clone of the current database URL string.
    pub async fn async_get_db_url(&self) -> String {
        let data = self.data.read().await;
        data.db_url.clone()
    }

    /// Stores the external plugin save directory asynchronously.
    ///
    /// # Arguments
    ///
    /// * `dir` - Optional directory path. If None, the getter will fall back to temp directory.
    ///
    /// # Returns
    ///
    /// Returns `()` once the directory has been written to the shared state.
    pub async fn async_set_ext_plugin_save_dir(&self, dir: Option<String>) {
        let mut data = self.data.write().await;
        data.ext_plugin_save_dir = dir;
    }

    /// Gets the external plugin save directory, falling back to system temp directory if not set.
    ///
    /// # Arguments
    ///
    /// This method takes no additional arguments beyond the borrowed [`GlobalState`].
    ///
    /// # Returns
    ///
    /// Returns the configured save directory or the system temp directory as fallback.
    pub async fn get_ext_plugin_save_dir(&self) -> String {
        let data = self.data.read().await;
        match &data.ext_plugin_save_dir {
            Some(dir) => dir.clone(),
            None => std::env::temp_dir().to_string_lossy().to_string(),
        }
    }

    /// Obtains the database URL by blocking within a Tokio-compatible context.
    ///
    /// # Arguments
    ///
    /// * `self` - An [`Arc`] handle to the global state used to access the shared data.
    ///
    /// # Returns
    ///
    /// Returns the persisted database URL string, blocking the current thread until the read completes.
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

    /// Updates the initialization flag asynchronously to reflect the database readiness state.
    ///
    /// # Arguments
    ///
    /// * `initialized` - Whether the database has been fully initialized.
    ///
    /// # Returns
    ///
    /// Returns `()` after the flag has been written.
    pub async fn async_set_db_initialized(&self, initialized: bool) {
        let mut data = self.data.write().await;
        data.db_initialized = initialized;
    }

    /// Spawns a task to update the database initialization flag.
    ///
    /// # Arguments
    ///
    /// * `self` - An [`Arc`] pointing to the shared state used inside the spawned task.
    /// * `initialized` - The new readiness value to persist.
    ///
    /// # Returns
    ///
    /// Returns immediately after spawning the asynchronous setter.
    pub fn set_db_initialized(self: std::sync::Arc<Self>, initialized: bool) {
        tokio::spawn(async move {
            self.async_set_db_initialized(initialized).await;
        });
    }

    /// Indicates whether the database has been marked as initialized, using a non-blocking read.
    ///
    /// # Arguments
    ///
    /// This method takes no additional arguments.
    ///
    /// # Returns
    ///
    /// Returns `true` when the initialization flag is set, or `false` if it is unset or currently locked.
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

    /// Waits for the database initialization flag to become `true`, logging progress as it polls.
    ///
    /// # Arguments
    ///
    /// This asynchronous method takes no additional arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once initialization completes, or an error when the timeout elapses.
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
    /// Renders the global state for debugging, falling back to a locked message when data is unavailable.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter receiving the textual representation.
    ///
    /// # Returns
    ///
    /// Returns `fmt::Result::Ok` when writing succeeds, or propagates formatter errors.
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

    /// Ensures the [`Display`] implementation surfaces default values when unlocked.
    ///
    /// # Arguments
    ///
    /// This test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after verifying the string representation contains the expected defaults.
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

    /// Confirms the [`Display`] implementation reports when the state is locked.
    ///
    /// # Arguments
    ///
    /// This test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after asserting the locked indicator appears in the formatted output.
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

    /// Validates that mutating the state is reflected by the [`Display`] output.
    ///
    /// # Arguments
    ///
    /// This test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after checking the formatter shows the updated values.
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

    /// Asserts `is_db_initialized` reflects flag changes and locked scenarios.
    ///
    /// # Arguments
    ///
    /// This test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after toggling the initialization flag under different locking conditions.
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

    /// Verifies `wait_db_initialized` exits immediately when the flag is already set.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after ensuring the wait call completes without timing out.
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

    /// Confirms `wait_db_initialized` blocks until another task sets the flag.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` once the spawned task flips the flag and the wait completes.
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

    /// Ensures multiple concurrent waiters unblock when initialization completes.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after verifying every wait task observes the flag change.
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

    /// Checks that the display output includes an updated database URL.
    ///
    /// # Arguments
    ///
    /// This test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after mutating the stored URL and formatting the state.
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

    /// Ensures the asynchronous setter flips the initialized flag immediately.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after asserting the flag is set to `true`.
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

    /// Validates the spawning setter eventually marks the database as initialized.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after polling until the spawned task updates the flag.
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

    /// Confirms the asynchronous getter returns the URL set via its paired setter.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after ensuring the stored URL matches the input value.
    #[tokio::test]
    async fn async_set_and_get_db_url_roundtrip() {
        let gs = GlobalState::new();

        // set the url asynchronously and then read it back
        gs.async_set_db_url("sqlite://async-test".to_string()).await;
        let got = gs.async_get_db_url().await;
        assert_eq!(got, "sqlite://async-test");
    }

    /// Verifies the blocking getter can be used safely from a non-async context.
    ///
    /// # Arguments
    ///
    /// This test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after reading the URL from a blocking context and confirming it matches.
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

    /// Ensures the spawning URL setter writes the value for subsequent asynchronous reads.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after polling until the spawned task updates the URL.
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

    /// Confirms `get_db_connection` succeeds when the URL and initialization flag are set.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after verifying a connection handle is obtained.
    #[tokio::test]
    async fn get_db_connection_returns_connection_when_url_set() {
        // Ensure that when a valid DB URL is present, get_db_connection returns a connection
        let gs = GlobalState::new();

        // set an in-memory sqlite URL
        {
            let mut w = gs.data.try_write().expect("should acquire write lock");
            w.db_url = "sqlite::memory:".to_string();
            w.db_initialized = true;
        }

        let conn = gs.get_db_connection().await;
        assert!(
            conn.is_ok(),
            "get_db_connection should return Ok when db_url is set"
        );
    }

    /// Ensures `wait_init_and_get_connection` blocks until setup completes and then returns a connection.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after confirming the helper waits and yields a valid handle.
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
