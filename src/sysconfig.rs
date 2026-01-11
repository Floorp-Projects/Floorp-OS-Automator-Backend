// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use sapphillon_core::plugin::{CorePluginPackage, PluginPackageTrait};
use sapphillon_core::proto::sapphillon::v1::PluginPackage;
use std::fmt;
use std::sync::Arc;

use crate::dummy_plugin::dummy_plugin_package;
use exec::{core_exec_plugin_package, exec_plugin_package};
use fetch::{core_fetch_plugin_package, fetch_plugin_package};
use filesystem::{core_filesystem_plugin_package, filesystem_plugin_package};
use search::{core_search_plugin_package, search_plugin_package};
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
#[allow(clippy::arc_with_non_send_sync)]
pub fn sysconfig() -> SysConfig {
    SysConfig {
        app_name: "Sapphillon",
        version: env!("CARGO_PKG_VERSION"),
        authors: env!("CARGO_PKG_AUTHORS"),
        copyright_year: 2025,

        core_plugin_package: vec![
            Arc::new(core_fetch_plugin_package()),
            Arc::new(core_filesystem_plugin_package()),
            Arc::new(core_search_plugin_package()),
            Arc::new(core_window_plugin_package()),
            Arc::new(core_exec_plugin_package()),
        ],
        initial_plugins: vec![
            fetch_plugin_package(),
            filesystem_plugin_package(),
            search_plugin_package(),
            window_plugin_package(),
            exec_plugin_package(),
            dummy_plugin_package(),
        ],

        initial_workflows: vec![],
    }
}

#[derive(Debug, Clone)]
pub struct InitialWorkflow {
    pub display_name: String,
    pub description: Option<String>,
    pub code: String,
}

#[derive(Clone)]
pub struct SysConfig {
    pub app_name: &'static str,
    pub version: &'static str,
    pub authors: &'static str,
    pub copyright_year: u16,

    pub core_plugin_package: Vec<Arc<dyn PluginPackageTrait>>,
    pub initial_plugins: Vec<PluginPackage>,

    pub initial_workflows: Vec<InitialWorkflow>,
}

impl fmt::Debug for SysConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SysConfig")
            .field("app_name", &self.app_name)
            .field("version", &self.version)
            .field("authors", &self.authors)
            .field("copyright_year", &self.copyright_year)
            .field(
                "core_plugin_package",
                &format_args!("[{} packages]", self.core_plugin_package.len()),
            )
            .field("initial_plugins", &self.initial_plugins)
            .field("initial_workflows", &self.initial_workflows)
            .finish()
    }
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
