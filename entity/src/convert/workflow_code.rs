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

use super::plugin_code::plugin_package_to_proto;
use crate::entity::permission::Model as EntityPermission;
use crate::entity::plugin_package::Model as EntityPluginPackage;
use crate::entity::workflow_code::Model as EntityWorkflowCode;
use crate::entity::workflow_code_allowed_permission::Model as EntityWCAllowed;
use sapphillon_core::proto::sapphillon::v1::WorkflowCode as ProtoWorkflowCode;
use sapphillon_core::proto::sapphillon::v1::WorkflowResult as ProtoWorkflowResult;
use sapphillon_core::proto::sapphillon::v1::{
    AllowedPermission as ProtoAllowedPermission, Permission as ProtoPermission,
};
use serde_json;
use std::collections::HashMap;

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
    use sapphillon_core::proto::sapphillon::v1::{PermissionLevel, PermissionType};

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
            Some(&[pkg.clone()]),
            Some(&["pf1".to_string()]),
            Some(&[(wc_allowed, Some(perm_entity))]),
        );

        assert_eq!(p.id, e.id);
        assert_eq!(p.plugin_packages.len(), 1);
        assert_eq!(p.plugin_function_ids.len(), 1);
        assert_eq!(p.allowed_permissions.len(), 1);
    }
}
