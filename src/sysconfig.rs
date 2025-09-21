use sapphillon_core::plugin::CorePluginPackage;
use sapphillon_core::proto::sapphillon::v1::PluginPackage;

use fetch::{core_fetch_plugin_package, fetch_plugin_package};
use filesystem::{core_filesystem_plugin_package, filesystem_plugin_package};

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

pub fn sysconfig() -> SysConfig {
    SysConfig {
        app_name: "Sapphillon",
        version: env!("CARGO_PKG_VERSION"),
        authors: env!("CARGO_PKG_AUTHORS"),
        copyright_year: 2025,

        core_plugin_package: vec![
            core_fetch_plugin_package(),
            core_filesystem_plugin_package(),
        ],
        plugin_package: vec![fetch_plugin_package(), filesystem_plugin_package()],
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
