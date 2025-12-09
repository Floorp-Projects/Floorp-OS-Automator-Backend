// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use sapphillon_core::plugin::CorePluginPackage;
use sapphillon_core::proto::sapphillon::v1::PluginPackage;

use exec::{core_exec_plugin_package, exec_plugin_package};
use fetch::{core_fetch_plugin_package, fetch_plugin_package};
use filesystem::{core_filesystem_plugin_package, filesystem_plugin_package};
use floorp::{core_floorp_plugin_package, floorp_plugin_package};
use search::{core_search_plugin_package, search_plugin_package};
use vscode::{core_vscode_plugin_package, vscode_plugin_package};
use window::{core_window_plugin_package, window_plugin_package};

/// Builds the static system configuration used during application startup.
///
/// # Arguments
///
/// This function takes no arguments.
///
/// # Returns
///
/// Returns a [`SysConfig`] populated with metadata and packaged plugins.
pub fn sysconfig() -> SysConfig {
    SysConfig {
        app_name: "Sapphillon",
        version: env!("CARGO_PKG_VERSION"),
        authors: env!("CARGO_PKG_AUTHORS"),
        copyright_year: 2025,

        core_plugin_package: vec![
            core_fetch_plugin_package(),
            core_filesystem_plugin_package(),
            core_floorp_plugin_package(),
            core_vscode_plugin_package(),
            core_search_plugin_package(),
            core_window_plugin_package(),
            core_exec_plugin_package(),
        ],
        plugin_package: vec![
            fetch_plugin_package(),
            filesystem_plugin_package(),
            floorp_plugin_package(),
            vscode_plugin_package(),
            search_plugin_package(),
            window_plugin_package(),
            exec_plugin_package(),
        ],
    }
}

#[derive(Debug, Clone)]
pub struct SysConfig {
    pub app_name: &'static str,
    pub version: &'static str,
    pub authors: &'static str,
    pub copyright_year: u16,

    pub core_plugin_package: Vec<CorePluginPackage>,
    pub plugin_package: Vec<PluginPackage>,
}

impl SysConfig {
    /// Formats human-readable application metadata for logs and console output.
    ///
    /// # Arguments
    ///
    /// * `self` - The configuration whose metadata should be rendered.
    ///
    /// # Returns
    ///
    /// Returns a multi-line string summarizing the application identity and licensing.
    pub fn app_info(&self) -> String {
        format!(
            "----------------------------------------\n\
            {} - Version: {}\n\
            Authors: {}\n\
            Copyright {} {}\n\
            \n\
            Made with Sapphillon\n\
            Licensed under the GNU General Public License v3.0 or later\n\
            https://github.com/Walkmana-25/Sapphillon\n\
            ----------------------------------------",
            self.app_name, self.version, self.authors, self.copyright_year, self.authors
        )
    }
}
