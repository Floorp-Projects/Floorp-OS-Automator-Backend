// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use sapphillon_core::proto::sapphillon::v1::{PluginFunction, PluginPackage};

pub fn dummy_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "*".to_string(),
        function_name: "Dummy Function".to_string(),
        version: "".to_string(),
        description: "A dummy function for wildcard permission.".to_string(),
        permissions: vec![],
        function_define: None,
    }
}

pub fn dummy_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.dummy".to_string(),
        package_name: "Dummy".to_string(),
        provider_id: "".to_string(),
        description: "A dummy plugin for wildcard permission.".to_string(),
        functions: vec![dummy_plugin_function()],
        package_version: "0.0.0".to_string(),
        deprecated: Some(true),
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}
