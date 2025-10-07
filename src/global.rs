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
}

