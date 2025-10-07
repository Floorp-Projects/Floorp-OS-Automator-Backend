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

use tokio::sync::RwLock;
use std::sync::LazyLock;

#[derive(Debug)]
pub struct GlobalStateData {
    db_initialized: bool,
    db_url: String
}

#[derive(Debug)]
pub struct GlobalState {
    pub data: LazyLock<RwLock<GlobalStateData>>,
}

impl GlobalState {
    pub const fn new() -> Self {
        GlobalState {
            data: LazyLock::new(|| RwLock::new(GlobalStateData {
                db_initialized: false,
                db_url: String::new(),
            })),
        }
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
        loop {
            let mut count = 1;
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
            Ok(data) => write!(f, "GlobalState {{ db_initialized: {}, db_url: '{}' }}", data.db_initialized, data.db_url),
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
        assert!(s.contains("db_initialized: false"), "display should show db_initialized: false, got: {s}");
        assert!(s.contains("db_url: ''"), "display should show empty db_url, got: {s}");
    }

    #[test]
    fn display_locked_shows_locked() {
        let gs = GlobalState::new();

        // Acquire an exclusive write lock so try_read inside Display fails
        let write_guard = gs.data.try_write().expect("should acquire write lock");
    let s = format!("{gs}");
    assert!(s.contains("<locked>"), "display should indicate locked state, got: {s}");

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
    assert!(s.contains("db_initialized: true"), "display should show db_initialized: true, got: {s}");
    assert!(s.contains("sqlite://:memory:"), "display should show updated db_url, got: {s}");
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
        assert!(gs.is_db_initialized(), "is_db_initialized should return true after setting the flag");

        // if we hold the write lock, try_read inside is_db_initialized will fail -> returns false
        let mut guard = gs.data.try_write().expect("should acquire write lock");
        guard.db_initialized = true;
        assert!(!gs.is_db_initialized(), "is_db_initialized should return false while locked (try_read fails)");
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
        assert!(res.is_ok(), "wait_db_initialized should return immediately when already initialized");
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
        assert!(res.is_ok(), "wait_db_initialized should return after the flag is set by another task");
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
        assert!(s.contains("postgres://localhost/mydb"), "display should show updated db_url, got: {s}");
    }
}

