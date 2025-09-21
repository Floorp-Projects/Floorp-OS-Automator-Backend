use crate::sysconfig;
use sapphillon_core::plugin::CorePluginPackage;
use sapphillon_core::proto::sapphillon::v1::PluginPackage;

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

pub const SYSCONFIG: SysConfig = SysConfig {
    app_name: "Sapphillon",
    version: env!("CARGO_PKG_VERSION"),
    authors: env!("CARGO_PKG_AUTHORS"),
    copyright_year: 2025,
};

#[derive(Debug, Clone)]
pub struct SysConfig {
    pub app_name: &'static str,
    pub version: &'static str,
    pub authors: &'static str,
    pub copyright_year: u16,
    

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
