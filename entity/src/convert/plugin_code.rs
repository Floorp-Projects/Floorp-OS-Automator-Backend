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

use crate::entity::permission::Model as EntityPermission;
use crate::entity::plugin_function::Model as EntityPluginFunction;
use crate::entity::plugin_package::Model as EntityPluginPackage;

use sapphillon_core::proto::sapphillon::v1::Permission as ProtoPermission;
use sapphillon_core::proto::sapphillon::v1::PluginFunction as ProtoPluginFunction;
use sapphillon_core::proto::sapphillon::v1::PluginPackage as ProtoPluginPackage;

/// Convert an entity `plugin_package::Model` into the proto `PluginPackage`.
/// This does not attach related `functions` by default; use the "with_relations"
/// variant when the caller has already loaded related records.
pub fn plugin_package_to_proto(entity: &EntityPluginPackage) -> ProtoPluginPackage {
    let installed_at =
        entity
            .installed_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            });

    let updated_at =
        entity
            .updated_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            });

    ProtoPluginPackage {
        package_id: entity.package_id.clone(),
        package_name: entity.package_name.clone(),
        package_version: entity.package_version.clone(),
        description: entity.description.clone().unwrap_or_default(),
        functions: Vec::new(),
        plugin_store_url: entity.plugin_store_url.clone().unwrap_or_default(),
        internal_plugin: Some(entity.internal_plugin),
        verified: Some(entity.verified),
        deprecated: Some(entity.deprecated),
        installed_at,
        updated_at,
    }
}

/// Like `plugin_package_to_proto` but allows attaching the function list when
/// the caller has already loaded related `plugin_function` records.
pub fn plugin_package_to_proto_with_functions(
    entity: &EntityPluginPackage,
    functions: Option<&[ProtoPluginFunction]>,
) -> ProtoPluginPackage {
    let mut p = plugin_package_to_proto(entity);
    if let Some(funcs) = functions {
        p.functions = funcs.to_vec();
    }
    p
}

/// Convert an entity `plugin_function::Model` into the proto `PluginFunction`.
/// Permissions may be attached when the caller provides them (loaded via relation).
pub fn plugin_function_to_proto(
    entity: &EntityPluginFunction,
    permissions: Option<&[ProtoPermission]>,
) -> ProtoPluginFunction {
    let args = entity.arguments.clone().unwrap_or_default();
    let ret = entity.returns.clone().unwrap_or_default();

    let mut p = ProtoPluginFunction {
        function_id: entity.function_id.clone(),
        function_name: entity.function_name.clone(),
        description: entity.description.clone().unwrap_or_default(),
        permissions: Vec::new(),
        arguments: args,
        returns: ret,
    };

    if let Some(perms) = permissions {
        p.permissions = perms.to_vec();
    }

    p
}

/// Convert an entity `permission::Model` into the proto `Permission` message.
pub fn permission_to_proto(entity: &EntityPermission) -> ProtoPermission {
    // Parse resource_json as Vec<String> when available, otherwise empty.
    let resource: Vec<String> = match &entity.resource_json {
        Some(s) => serde_json::from_str(s).unwrap_or_default(),
        None => Vec::new(),
    };

    let level = entity
        .level
        .unwrap_or(sapphillon_core::proto::sapphillon::v1::PermissionLevel::Unspecified as i32);

    ProtoPermission {
        display_name: entity.display_name.clone().unwrap_or_default(),
        description: entity.description.clone().unwrap_or_default(),
        permission_type: entity.r#type,
        resource,
        permission_level: level,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::permission::Model as EntityPermission;
    use crate::entity::plugin_function::Model as EntityPluginFunction;
    use crate::entity::plugin_package::Model as EntityPluginPackage;
    

    #[test]
    fn converts_minimal_package() {
        let e = EntityPluginPackage {
            package_id: "pkg1".to_string(),
            package_name: "P".to_string(),
            package_version: "v1".to_string(),
            description: None,
            plugin_store_url: None,
            internal_plugin: true,
            verified: true,
            deprecated: false,
            installed_at: None,
            updated_at: None,
        };

        let p = plugin_package_to_proto(&e);

        assert_eq!(p.package_id, e.package_id);
        assert_eq!(p.package_version, e.package_version);
        assert_eq!(p.package_name, e.package_name);
        assert_eq!(p.functions.len(), 0);
        assert_eq!(p.internal_plugin, Some(true));
    }

    #[test]
    fn converts_function_and_permission() {
        let f = EntityPluginFunction {
            function_id: "pkg.fn".to_string(),
            package_id: "pkg1".to_string(),
            function_name: "Fn".to_string(),
            description: Some("do it".to_string()),
            arguments: Some("{}".to_string()),
            returns: Some("{}".to_string()),
        };

        let perm_entity = EntityPermission {
            id: 1,
            plugin_function_id: "pkg.fn".to_string(),
            display_name: Some("Read".to_string()),
            description: Some("desc".to_string()),
            r#type: sapphillon_core::proto::sapphillon::v1::PermissionType::FilesystemRead as i32,
            resource_json: Some("[\"secrets/x\"]".to_string()),
            level: Some(
                sapphillon_core::proto::sapphillon::v1::PermissionLevel::Unspecified as i32,
            ),
        };

        let proto_perm = permission_to_proto(&perm_entity);
        assert_eq!(proto_perm.display_name, "Read");
        assert_eq!(proto_perm.resource.len(), 1);

        let proto_fn = plugin_function_to_proto(&f, Some(&[proto_perm.clone()]));
        assert_eq!(proto_fn.function_id, f.function_id);
        assert_eq!(proto_fn.permissions.len(), 1);
    }
}
