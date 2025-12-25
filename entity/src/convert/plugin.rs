// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! This module provides functions for converting between the plugin-related entities and
//! their corresponding protobuf representations.

use crate::entity::permission::Model as EntityPermission;
use crate::entity::plugin_function::Model as EntityPluginFunction;
use crate::entity::plugin_package::Model as EntityPluginPackage;

use sapphillon_core::proto::sapphillon::v1::FunctionDefine;
use sapphillon_core::proto::sapphillon::v1::FunctionParameter;
use sapphillon_core::proto::sapphillon::v1::Permission as ProtoPermission;
use sapphillon_core::proto::sapphillon::v1::PluginFunction as ProtoPluginFunction;
use sapphillon_core::proto::sapphillon::v1::PluginPackage as ProtoPluginPackage;
use serde::{Deserialize, Serialize};

/// Convert an entity `plugin_package::Model` into the proto `PluginPackage`.
/// This does not attach related `functions` by default; use the "with_relations"
/// variant when the caller has already loaded related `plugin_function` records.
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
    let function_define = parse_function_define_from_entity(entity);

    let mut p = ProtoPluginFunction {
        function_id: entity.function_id.clone(),
        function_name: entity.function_name.clone(),
        description: entity.description.clone().unwrap_or_default(),
        permissions: Vec::new(),
        function_define,
    };

    if let Some(perms) = permissions {
        p.permissions = perms.to_vec();
    }

    p
}

fn parse_function_define_from_entity(entity: &EntityPluginFunction) -> Option<FunctionDefine> {
    let parameters = entity
        .arguments
        .as_deref()
        .and_then(parse_params_json)
        .unwrap_or_default();

    let returns = entity
        .returns
        .as_deref()
        .and_then(parse_params_json)
        .unwrap_or_default();

    if parameters.is_empty() && returns.is_empty() {
        None
    } else {
        Some(FunctionDefine {
            parameters,
            returns,
        })
    }
}

fn parse_params_json(raw: &str) -> Option<Vec<FunctionParameter>> {
    serde_json::from_str::<Vec<FunctionParameterSerde>>(raw)
        .ok()
        .map(|params| params.into_iter().map(FunctionParameter::from).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionParameterSerde {
    name: String,
    r#type: String,
    description: String,
}

impl From<FunctionParameterSerde> for FunctionParameter {
    fn from(value: FunctionParameterSerde) -> Self {
        FunctionParameter {
            name: value.name,
            r#type: value.r#type,
            description: value.description,
        }
    }
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
    use sapphillon_core::proto::google::protobuf::Timestamp;
    use sapphillon_core::proto::sapphillon::v1::{PermissionLevel, PermissionType};

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
            arguments: Some(
                serde_json::to_string(&vec![FunctionParameterSerde {
                    name: "path".to_string(),
                    r#type: "string".to_string(),
                    description: "file path".to_string(),
                }])
                .unwrap(),
            ),
            returns: Some(
                serde_json::to_string(&vec![FunctionParameterSerde {
                    name: "result".to_string(),
                    r#type: "string".to_string(),
                    description: "result".to_string(),
                }])
                .unwrap(),
            ),
        };

        let perm_entity = EntityPermission {
            id: 1,
            plugin_function_id: "pkg.fn".to_string(),
            display_name: Some("Read".to_string()),
            description: Some("desc".to_string()),
            r#type: PermissionType::FilesystemRead as i32,
            resource_json: Some("[\"secrets/x\"]".to_string()),
            level: Some(PermissionLevel::Unspecified as i32),
        };

        let proto_perm = permission_to_proto(&perm_entity);
        assert_eq!(proto_perm.display_name, "Read");
        assert_eq!(proto_perm.resource.len(), 1);

        let proto_fn = plugin_function_to_proto(&f, Some(std::slice::from_ref(&proto_perm)));
        assert_eq!(proto_fn.function_id, f.function_id);
        assert_eq!(proto_fn.permissions.len(), 1);
        let fd = proto_fn.function_define.unwrap();
        assert_eq!(fd.parameters.len(), 1);
        assert_eq!(fd.returns.len(), 1);
    }

    #[test]
    fn converts_proto_package_to_entity_roundtrip_fields() {
        let proto = ProtoPluginPackage {
            package_id: "pkg1".to_string(),
            package_name: "Plugin".to_string(),
            package_version: "1.2.3".to_string(),
            description: "Best plugin".to_string(),
            functions: Vec::new(),
            plugin_store_url: "https://example.com".to_string(),
            internal_plugin: Some(true),
            verified: Some(true),
            deprecated: Some(false),
            installed_at: Some(Timestamp {
                seconds: 1_726_000_000,
                nanos: 123_000_000,
            }),
            updated_at: Some(Timestamp {
                seconds: 1_726_000_001,
                nanos: 987_000_000,
            }),
        };

        // We only have entity->proto helpers here; ensure plugin_package_to_proto consumes entity fields correctly
        // Build an entity from proto-like values to test mapping in the entity->proto direction.
        let entity = EntityPluginPackage {
            package_id: proto.package_id.clone(),
            package_name: proto.package_name.clone(),
            package_version: proto.package_version.clone(),
            description: Some(proto.description.clone()),
            plugin_store_url: Some(proto.plugin_store_url.clone()),
            internal_plugin: proto.internal_plugin.unwrap_or(false),
            verified: proto.verified.unwrap_or(false),
            deprecated: proto.deprecated.unwrap_or(false),
            installed_at: None,
            updated_at: None,
        };

        let out = plugin_package_to_proto(&entity);
        assert_eq!(out.package_id, entity.package_id);
        assert_eq!(out.package_name, entity.package_name);
        assert_eq!(out.package_version, entity.package_version);
        assert_eq!(
            out.description,
            entity.description.clone().unwrap_or_default()
        );
        assert_eq!(
            out.plugin_store_url,
            entity.plugin_store_url.clone().unwrap_or_default()
        );
    }
}
