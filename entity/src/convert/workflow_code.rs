// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! This module provides functions for converting between the `workflow_code` and
//! related entities and their corresponding protobuf representations.

use super::plugin_code::{
    plugin_package_to_proto, proto_string_to_option, proto_timestamp_to_datetime,
    proto_to_permission,
};
use crate::entity::permission::Model as EntityPermission;
use crate::entity::plugin_package::Model as EntityPluginPackage;
use crate::entity::workflow_code::Model as EntityWorkflowCode;
use crate::entity::workflow_code_allowed_permission::Model as EntityWCAllowed;
use crate::entity::workflow_code_plugin_function::Model as EntityWCPluginFunction;
use crate::entity::workflow_code_plugin_package::Model as EntityWCPluginPackage;
use crate::entity::workflow_result::Model as EntityWorkflowResult;
use sapphillon_core::proto::sapphillon::v1::PluginPackage as ProtoPluginPackage;
use sapphillon_core::proto::sapphillon::v1::WorkflowCode as ProtoWorkflowCode;
use sapphillon_core::proto::sapphillon::v1::WorkflowResult as ProtoWorkflowResult;
use sapphillon_core::proto::sapphillon::v1::{
    AllowedPermission as ProtoAllowedPermission, Permission as ProtoPermission,
};
use serde_json;
use std::collections::HashMap;

/// Convert a protobuf `WorkflowCode` into the entity model.
///
/// The caller must provide the owning `workflow_id`, since it does not appear in the
/// proto representation. Timestamps are normalized using
/// `proto_timestamp_to_datetime` to avoid panics on invalid ranges.
pub fn proto_to_workflow_code(
    proto: &ProtoWorkflowCode,
    workflow_id: impl Into<String>,
) -> EntityWorkflowCode {
    let created_at = proto
        .created_at
        .as_ref()
        .and_then(proto_timestamp_to_datetime);

    EntityWorkflowCode {
        id: proto.id.clone(),
        workflow_id: workflow_id.into(),
        code_revision: proto.code_revision,
        code: proto.code.clone(),
        language: proto.language,
        created_at,
    }
}

/// Convert proto plugin package references into join-table entities that link a
/// workflow code to the packages it depends on. The primary key is set to zero so the
/// caller can insert new rows without conflicting with existing IDs.
pub fn proto_to_workflow_code_plugin_packages(
    workflow_code_id: impl Into<String>,
    packages: &[ProtoPluginPackage],
) -> Vec<EntityWCPluginPackage> {
    let workflow_code_id = workflow_code_id.into();

    packages
        .iter()
        .map(|pkg| EntityWCPluginPackage {
            id: 0,
            workflow_code_id: workflow_code_id.clone(),
            plugin_package_id: pkg.package_id.clone(),
        })
        .collect()
}

/// Convert proto plugin function identifiers into join-table entities relating them to
/// the workflow code.
pub fn proto_to_workflow_code_plugin_functions(
    workflow_code_id: impl Into<String>,
    plugin_function_ids: &[String],
) -> Vec<EntityWCPluginFunction> {
    let workflow_code_id = workflow_code_id.into();

    plugin_function_ids
        .iter()
        .map(|function_id| EntityWCPluginFunction {
            id: 0,
            workflow_code_id: workflow_code_id.clone(),
            plugin_function_id: function_id.clone(),
        })
        .collect()
}

/// Flatten proto `AllowedPermission` messages into pairs of permission entities and
/// their join-table counterparts. Permission IDs remain at the default `0` so callers
/// can insert new records and update the relations accordingly.
pub fn proto_allowed_permissions_to_entities(
    workflow_code_id: impl Into<String>,
    allowed_permissions: &[ProtoAllowedPermission],
) -> Vec<(EntityWCAllowed, EntityPermission)> {
    let workflow_code_id = workflow_code_id.into();
    let mut out = Vec::new();

    for allowed in allowed_permissions {
        let function_id = allowed.plugin_function_id.clone();
        for perm_proto in &allowed.permissions {
            let permission = proto_to_permission(perm_proto, function_id.clone(), None);
            let relation = EntityWCAllowed {
                id: 0,
                workflow_code_id: workflow_code_id.clone(),
                permission_id: permission.id,
            };
            out.push((relation, permission));
        }
    }

    out
}

/// Convert a proto `WorkflowResult` into the entity model using the supplied workflow
/// identifiers.
pub fn proto_to_workflow_result(
    proto: &ProtoWorkflowResult,
    workflow_id: impl Into<String>,
    workflow_code_id: impl Into<String>,
) -> EntityWorkflowResult {
    let ran_at = proto.ran_at.as_ref().and_then(proto_timestamp_to_datetime);

    EntityWorkflowResult {
        id: proto.id.clone(),
        workflow_id: workflow_id.into(),
        workflow_code_id: workflow_code_id.into(),
        display_name: proto_string_to_option(&proto.display_name),
        description: proto_string_to_option(&proto.description),
        result: proto_string_to_option(&proto.result),
        ran_at,
        result_type: proto.result_type,
        exit_code: Some(proto.exit_code),
        workflow_result_revision: proto.workflow_result_revision,
    }
}

/// Convert an entity `workflow_code::Model` into the corresponding
/// proto `WorkflowCode` message.
///
/// This is intentionally a plain function (not an Into/From impl) so
/// callers can invoke an explicit conversion without relying on trait
/// resolution or implicit conversions.
pub fn workflow_code_to_proto(entity: &EntityWorkflowCode) -> ProtoWorkflowCode {
    // Map optional created_at (chrono::DateTime<Utc>) to protobuf Timestamp
    let created_at =
        entity
            .created_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            });

    ProtoWorkflowCode {
        id: entity.id.clone(),
        // workflow_id is stored in the DB entity but the proto message for
        // WorkflowCode intentionally omits it; keep fields consistent with
        // other conversions in the codebase.
        code_revision: entity.code_revision,
        code: entity.code.clone(),
        language: entity.language,
        created_at,
        // The following fields are left empty by default; callers may
        // populate them separately when they have joined/loaded relations.
        result: Vec::new(),
        plugin_packages: Vec::new(),
        plugin_function_ids: Vec::new(),
        allowed_permissions: Vec::new(),
    }
}

/// Convert an entity `workflow_code::Model` into the corresponding proto
/// `WorkflowCode` message, optionally attaching relation vectors when the
/// caller has already loaded related records.
pub fn workflow_code_to_proto_with_relations(
    entity: &EntityWorkflowCode,
    result: Option<&[ProtoWorkflowResult]>,
    plugin_packages: Option<&[EntityPluginPackage]>,
    plugin_function_ids: Option<&[String]>,
    allowed_permissions: Option<&[(EntityWCAllowed, Option<EntityPermission>)]>,
) -> ProtoWorkflowCode {
    let mut p = workflow_code_to_proto(entity);

    if let Some(r) = result {
        p.result = r.to_vec();
    }

    if let Some(pp_entities) = plugin_packages {
        // convert entity plugin packages into proto messages
        p.plugin_packages = pp_entities.iter().map(plugin_package_to_proto).collect();
    }

    if let Some(pf_ids) = plugin_function_ids {
        p.plugin_function_ids = pf_ids.to_vec();
    }

    if let Some(ap_entities) = allowed_permissions {
        // convert entity allowed-permission relation tuples into proto grouping
        p.allowed_permissions = allowed_permissions_to_proto(ap_entities);
    }

    p
}

/// Convert DB allowed permission relations into the protobuf AllowedPermission
/// grouping by plugin_function_id. The input is a slice of tuples where the
/// second element is the optional related permission record. This mirrors the
/// return shape of the CRUD helper that loads the relation plus permission.
pub fn allowed_permissions_to_proto(
    items: &[(EntityWCAllowed, Option<EntityPermission>)],
) -> Vec<ProtoAllowedPermission> {
    let mut map: HashMap<String, Vec<ProtoPermission>> = HashMap::new();

    for (_rel, perm_opt) in items.iter() {
        let perm = match perm_opt {
            Some(p) => p,
            None => continue, // no permission row; skip
        };

        // Parse resource_json (stored as JSON array of strings) into Vec<String>
        let resources: Vec<String> = match &perm.resource_json {
            Some(s) => serde_json::from_str::<Vec<String>>(s).unwrap_or_else(|_| Vec::new()),
            None => Vec::new(),
        };

        let proto_perm = ProtoPermission {
            display_name: perm.display_name.clone().unwrap_or_default(),
            description: perm.description.clone().unwrap_or_default(),
            permission_type: perm.r#type,
            resource: resources,
            permission_level: perm.level.unwrap_or_default(),
        };

        map.entry(perm.plugin_function_id.clone())
            .or_default()
            .push(proto_perm);
    }

    // Convert hashmap into Vec<AllowedPermission>
    let mut out: Vec<ProtoAllowedPermission> = map
        .into_iter()
        .map(|(plugin_function_id, permissions)| ProtoAllowedPermission {
            plugin_function_id,
            permissions,
        })
        .collect();

    // Keep ordering deterministic: sort by plugin_function_id
    out.sort_by(|a, b| a.plugin_function_id.cmp(&b.plugin_function_id));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::permission::Model as EntityPermission;
    use crate::entity::plugin_package::Model as EntityPluginPackage;
    use crate::entity::workflow_code::Model as EntityWorkflowCode;
    use crate::entity::workflow_code_allowed_permission::Model as EntityWCAllowed;
    use sapphillon_core::proto::google::protobuf::Timestamp;
    use sapphillon_core::proto::sapphillon::v1::{PermissionLevel, PermissionType};
    use sapphillon_core::proto::sapphillon::v1::{
        PluginPackage as ProtoPluginPackage, WorkflowResultType,
    };

    #[test]
    fn converts_minimal_entity_to_proto() {
        let e = EntityWorkflowCode {
            id: "wc1".to_string(),
            workflow_id: "wf1".to_string(),
            code_revision: 1,
            code: "print('hi')".to_string(),
            language: 0,
            created_at: None,
        };

        let p = workflow_code_to_proto(&e);

        assert_eq!(p.id, e.id);
        assert_eq!(p.code_revision, e.code_revision);
        assert_eq!(p.code, e.code);
        assert_eq!(p.language, e.language);
        assert!(p.created_at.is_none());
    }

    #[test]
    fn converts_with_relations_attached() {
        let e = EntityWorkflowCode {
            id: "wc2".to_string(),
            workflow_id: "wf2".to_string(),
            code_revision: 2,
            code: "print('bye')".to_string(),
            language: 1,
            created_at: None,
        };

        let pkg = EntityPluginPackage {
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

        let wc_allowed = EntityWCAllowed {
            id: 1,
            workflow_code_id: e.id.clone(),
            permission_id: 1,
        };

        let perm_entity = EntityPermission {
            id: 1,
            plugin_function_id: "pf1".to_string(),
            display_name: Some("X".to_string()),
            description: Some("D".to_string()),
            r#type: PermissionType::FilesystemRead as i32,
            resource_json: None,
            level: Some(PermissionLevel::Unspecified as i32),
        };

        let p = workflow_code_to_proto_with_relations(
            &e,
            None,
            Some(std::slice::from_ref(&pkg)),
            Some(&["pf1".to_string()]),
            Some(&[(wc_allowed, Some(perm_entity))]),
        );

        assert_eq!(p.id, e.id);
        assert_eq!(p.plugin_packages.len(), 1);
        assert_eq!(p.plugin_function_ids.len(), 1);
        assert_eq!(p.allowed_permissions.len(), 1);
    }

    #[test]
    fn converts_proto_workflow_code_to_entity() {
        let proto = ProtoWorkflowCode {
            id: "wc-id".to_string(),
            code_revision: 5,
            code: "console.log('hi')".to_string(),
            language: 1,
            created_at: Some(Timestamp {
                seconds: 1_726_000_000,
                nanos: 111_000_000,
            }),
            result: Vec::new(),
            plugin_packages: Vec::new(),
            plugin_function_ids: Vec::new(),
            allowed_permissions: Vec::new(),
        };

        let entity = proto_to_workflow_code(&proto, "wf-id");

        assert_eq!(entity.id, proto.id);
        assert_eq!(entity.workflow_id, "wf-id");
        assert_eq!(entity.code_revision, proto.code_revision);
        assert_eq!(entity.code, proto.code);
        assert_eq!(entity.language, proto.language);
        assert_eq!(
            entity.created_at.unwrap().timestamp(),
            proto.created_at.as_ref().unwrap().seconds
        );
    }

    #[test]
    fn converts_proto_plugin_relations() {
        let packages = vec![ProtoPluginPackage {
            package_id: "pkg1".to_string(),
            package_name: "Pkg".to_string(),
            package_version: "1.0.0".to_string(),
            description: String::new(),
            functions: Vec::new(),
            plugin_store_url: String::new(),
            internal_plugin: None,
            verified: None,
            deprecated: None,
            installed_at: None,
            updated_at: None,
        }];

        let package_links = proto_to_workflow_code_plugin_packages("wc", &packages);
        assert_eq!(package_links.len(), 1);
        assert_eq!(package_links[0].workflow_code_id, "wc");
        assert_eq!(package_links[0].plugin_package_id, "pkg1");
        assert_eq!(package_links[0].id, 0);

        let functions = vec!["pkg.fn".to_string(), "pkg.fn2".to_string()];
        let function_links = proto_to_workflow_code_plugin_functions("wc", &functions);
        assert_eq!(function_links.len(), 2);
        assert!(
            function_links
                .iter()
                .all(|link| link.workflow_code_id == "wc" && link.id == 0)
        );
        assert_eq!(function_links[0].plugin_function_id, "pkg.fn");
        assert_eq!(function_links[1].plugin_function_id, "pkg.fn2");
    }

    #[test]
    fn converts_proto_allowed_permissions_to_entities() {
        let proto_permission = ProtoPermission {
            display_name: "Read".to_string(),
            description: "Allow read".to_string(),
            permission_type: PermissionType::FilesystemRead as i32,
            resource: vec!["secrets/path".to_string()],
            permission_level: PermissionLevel::High as i32,
        };

        let allowed = ProtoAllowedPermission {
            plugin_function_id: "pkg.fn".to_string(),
            permissions: vec![proto_permission.clone()],
        };

        let tuples = proto_allowed_permissions_to_entities("wc", &[allowed]);
        assert_eq!(tuples.len(), 1);

        let (relation, permission) = &tuples[0];
        assert_eq!(relation.workflow_code_id, "wc");
        assert_eq!(relation.permission_id, permission.id);
        assert_eq!(permission.plugin_function_id, "pkg.fn");
        assert_eq!(permission.r#type, proto_permission.permission_type);
        assert_eq!(permission.level, Some(proto_permission.permission_level));
    }

    #[test]
    fn converts_proto_workflow_result_to_entity() {
        let proto = ProtoWorkflowResult {
            id: "res1".to_string(),
            display_name: "Result".to_string(),
            description: String::new(),
            result: String::new(),
            ran_at: Some(Timestamp {
                seconds: 1_726_500_000,
                nanos: 0,
            }),
            result_type: WorkflowResultType::SuccessUnspecified as i32,
            exit_code: 0,
            workflow_result_revision: 3,
        };

        let entity = proto_to_workflow_result(&proto, "wf", "wc");

        assert_eq!(entity.id, proto.id);
        assert_eq!(entity.workflow_id, "wf");
        assert_eq!(entity.workflow_code_id, "wc");
        assert_eq!(entity.display_name.as_deref(), Some("Result"));
        assert!(entity.description.is_none());
        assert!(entity.result.is_none());
        assert_eq!(
            entity.ran_at.unwrap().timestamp(),
            proto.ran_at.as_ref().unwrap().seconds
        );
        assert_eq!(entity.result_type, proto.result_type);
        assert_eq!(entity.exit_code, Some(0));
        assert_eq!(
            entity.workflow_result_revision,
            proto.workflow_result_revision
        );
    }
}
