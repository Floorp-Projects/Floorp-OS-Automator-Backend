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

use chrono::{TimeZone, Utc};
use sea_orm::prelude::DateTimeUtc;

use sapphillon_core::proto::sapphillon::v1::Permission as ProtoPermission;
use sapphillon_core::proto::sapphillon::v1::PluginFunction as ProtoPluginFunction;
use sapphillon_core::proto::sapphillon::v1::PluginPackage as ProtoPluginPackage;

/// Convert a protobuf `PluginPackage` into the entity model.
///
/// The returned entity only contains scalar fields present on the package table.
/// Related records (e.g. plugin functions) must be handled separately by the caller.
pub fn proto_to_plugin_package(proto: &ProtoPluginPackage) -> EntityPluginPackage {
    let installed_at = proto
        .installed_at
        .as_ref()
        .and_then(proto_timestamp_to_datetime);

    let updated_at = proto
        .updated_at
        .as_ref()
        .and_then(proto_timestamp_to_datetime);

    EntityPluginPackage {
        package_id: proto.package_id.clone(),
        package_name: proto.package_name.clone(),
        package_version: proto.package_version.clone(),
        description: proto_string_to_option(&proto.description),
        plugin_store_url: proto_string_to_option(&proto.plugin_store_url),
        internal_plugin: proto.internal_plugin.unwrap_or(false),
        verified: proto.verified.unwrap_or(false),
        deprecated: proto.deprecated.unwrap_or(false),
        installed_at,
        updated_at,
    }
}

/// Convert a protobuf `PluginFunction` into the entity model.
///
/// The caller must supply the owning `package_id`, since this field is not included
/// in the protobuf message. Optional string fields are converted into `Option<String>`
/// by treating the empty string as `None`.
pub fn proto_to_plugin_function(
    proto: &ProtoPluginFunction,
    package_id: impl Into<String>,
) -> EntityPluginFunction {
    EntityPluginFunction {
        function_id: proto.function_id.clone(),
        package_id: package_id.into(),
        function_name: proto.function_name.clone(),
        description: proto_string_to_option(&proto.description),
        arguments: proto_string_to_option(&proto.arguments),
        returns: proto_string_to_option(&proto.returns),
    }
}

/// Convert a protobuf `Permission` into the entity model.
///
/// The caller must pass the associated `plugin_function_id`; the database `id` can be
/// provided when known (use `None` when constructing a new record before insertion).
/// Permission resources are re-encoded as JSON, mirroring how they are stored in the
/// entity layer. `PermissionLevel::Unspecified` is mapped to `None` to avoid persisting
/// redundant default values.
pub fn proto_to_permission(
    proto: &ProtoPermission,
    plugin_function_id: impl Into<String>,
    id: Option<i32>,
) -> EntityPermission {
    let resource_json = if proto.resource.is_empty() {
        None
    } else {
        serde_json::to_string(&proto.resource).ok()
    };

    let level_value = proto.permission_level;
    let level = if level_value
        == sapphillon_core::proto::sapphillon::v1::PermissionLevel::Unspecified as i32
    {
        None
    } else {
        Some(level_value)
    };

    EntityPermission {
        id: id.unwrap_or_default(),
        plugin_function_id: plugin_function_id.into(),
        display_name: proto_string_to_option(&proto.display_name),
        description: proto_string_to_option(&proto.description),
        r#type: proto.permission_type,
        resource_json,
        level,
    }
}

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

fn proto_string_to_option(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn proto_timestamp_to_datetime(
    ts: &sapphillon_core::proto::google::protobuf::Timestamp,
) -> Option<DateTimeUtc> {
    let mut seconds = ts.seconds;
    let mut nanos = ts.nanos;

    if nanos < 0 {
        let adjustment = ((-nanos) as i64 + 999_999_999) / 1_000_000_000;
        seconds = seconds.checked_sub(adjustment)?;
        nanos += (adjustment * 1_000_000_000) as i32;
    } else if nanos >= 1_000_000_000 {
        let adjustment = nanos / 1_000_000_000;
        seconds = seconds.checked_add(adjustment as i64)?;
        nanos %= 1_000_000_000;
    }

    if !(0..1_000_000_000).contains(&nanos) {
        return None;
    }

    Utc.timestamp_opt(seconds, nanos as u32).single()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::permission::Model as EntityPermission;
    use crate::entity::plugin_function::Model as EntityPluginFunction;
    use crate::entity::plugin_package::Model as EntityPluginPackage;
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

    #[test]
    fn converts_proto_package_to_entity() {
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

        let entity = proto_to_plugin_package(&proto);

        assert_eq!(entity.package_id, proto.package_id);
        assert_eq!(entity.package_name, proto.package_name);
        assert_eq!(entity.package_version, proto.package_version);
        assert_eq!(entity.description.as_deref(), Some("Best plugin"));
        assert_eq!(
            entity.plugin_store_url.as_deref(),
            Some("https://example.com")
        );
        assert!(entity.internal_plugin);
        assert!(entity.verified);
        assert!(!entity.deprecated);
        assert_eq!(
            entity.installed_at.unwrap().timestamp(),
            proto.installed_at.as_ref().unwrap().seconds
        );
        assert_eq!(
            entity.updated_at.unwrap().timestamp(),
            proto.updated_at.as_ref().unwrap().seconds
        );
    }

    #[test]
    fn converts_proto_function_to_entity() {
        let proto = ProtoPluginFunction {
            function_id: "pkg.fn".to_string(),
            function_name: "Fn".to_string(),
            description: "do it".to_string(),
            permissions: Vec::new(),
            arguments: "{}".to_string(),
            returns: "{}".to_string(),
        };

        let entity = proto_to_plugin_function(&proto, "pkg1");

        assert_eq!(entity.function_id, proto.function_id);
        assert_eq!(entity.function_name, proto.function_name);
        assert_eq!(entity.package_id, "pkg1");
        assert_eq!(entity.description.as_deref(), Some("do it"));
        assert_eq!(entity.arguments.as_deref(), Some("{}"));
        assert_eq!(entity.returns.as_deref(), Some("{}"));
    }

    #[test]
    fn converts_proto_permission_to_entity() {
        let proto = ProtoPermission {
            display_name: "Read".to_string(),
            description: "desc".to_string(),
            permission_type: PermissionType::FilesystemRead as i32,
            resource: vec!["secrets/x".to_string()],
            permission_level: PermissionLevel::Medium as i32,
        };

        let entity = proto_to_permission(&proto, "pkg.fn", Some(42));

        assert_eq!(entity.id, 42);
        assert_eq!(entity.plugin_function_id, "pkg.fn");
        assert_eq!(entity.display_name.as_deref(), Some("Read"));
        assert_eq!(entity.description.as_deref(), Some("desc"));
        assert_eq!(entity.r#type, PermissionType::FilesystemRead as i32);
        assert!(entity.resource_json.unwrap().contains("secrets/x"));
        assert_eq!(entity.level, Some(PermissionLevel::Medium as i32));
    }
}
