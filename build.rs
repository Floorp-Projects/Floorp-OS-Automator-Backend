// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO Re-enable Windows support
    #[cfg(target_os = "windows")]
    compile_error!("Currently, Windows support is suspended.");

    Ok(())
}
