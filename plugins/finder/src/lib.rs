// Finder plugin - lightweight wrapper that composes Search + Filesystem + Exec
// SPDX-FileCopyrightText: 2026 GitHub Copilot
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{PluginFunction, PluginPackage};

pub fn finder_noop_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.finder.noop".to_string(),
        function_name: "Finder Noop".to_string(),
        version: "".to_string(),
        description: "Simple noop for Finder plugin registration".to_string(),
        permissions: vec![],
        function_define: None,
    }
}

pub fn finder_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.finder".to_string(),
        package_name: "Finder".to_string(),
        provider_id: "".to_string(),
        description: "A simple Finder helper plugin that composes search + filesystem + exec to collect local files and extract text.".to_string(),
        functions: vec![finder_noop_plugin_function()],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_finder_noop() -> CorePluginFunction {
    CorePluginFunction::new(
        finder_noop_plugin_function().function_id,
        "Finder Noop".to_string(),
        finder_noop_plugin_function().description,
        op2_finder_noop(),
        Some(include_str!("00_finder.js").to_string()),
    )
}

pub fn core_finder_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        finder_plugin_package().package_id,
        "Finder".to_string(),
        vec![core_finder_noop()],
    )
}

#[op2]
#[string]
fn op2_finder_noop(_state: &mut OpState) -> Result<String, JsErrorBox> {
    Ok("ok".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_function_exists() {
        // Verify that the noop function can be constructed
        let _noop = op2_finder_noop;
    }
}
