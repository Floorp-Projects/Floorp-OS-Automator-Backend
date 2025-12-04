// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! This module provides functions for converting between the `provider` entity and its
//! corresponding protobuf representation.

use crate::entity::provider::Model as EntityProvider;
use sapphillon_core::proto::sapphillon::ai::v1::Provider as ProtoProvider;

impl From<EntityProvider> for ProtoProvider {
    fn from(entity: EntityProvider) -> Self {
        ProtoProvider {
            name: entity.name,
            display_name: entity.display_name,
            api_key: entity.api_key,
            api_endpoint: entity.api_endpoint,
        }
    }
}

impl From<ProtoProvider> for EntityProvider {
    fn from(proto: ProtoProvider) -> Self {
        EntityProvider {
            name: proto.name,
            display_name: proto.display_name,
            api_key: proto.api_key,
            api_endpoint: proto.api_endpoint,
        }
    }
}

impl From<&EntityProvider> for ProtoProvider {
    fn from(entity: &EntityProvider) -> Self {
        entity.clone().into()
    }
}

impl From<&ProtoProvider> for EntityProvider {
    fn from(proto: &ProtoProvider) -> Self {
        proto.clone().into()
    }
}

/// Converts a provider entity reference into its proto representation.
pub fn provider_entity_to_proto(entity: &EntityProvider) -> ProtoProvider {
    entity.into()
}

/// Converts a provider proto reference into the corresponding entity model.
pub fn provider_proto_to_entity(proto: &ProtoProvider) -> EntityProvider {
    proto.into()
}

/// Convenience helper to map a slice of provider entities into proto messages.
pub fn provider_entities_to_proto(entities: &[EntityProvider]) -> Vec<ProtoProvider> {
    entities.iter().map(provider_entity_to_proto).collect()
}

#[cfg(test)]
mod tests {
    use super::{provider_entities_to_proto, provider_entity_to_proto, provider_proto_to_entity};
    use crate::entity::provider::Model as EntityProvider;
    use sapphillon_core::proto::sapphillon::ai::v1::Provider as ProtoProvider;

    #[test]
    fn entity_to_proto_and_back() {
        let entity = EntityProvider {
            name: "providers/my_provider".to_string(),
            display_name: "My Provider".to_string(),
            api_key: "secret-key".to_string(),
            api_endpoint: "https://api.example.com".to_string(),
        };

        // Entity -> Proto
        let proto: ProtoProvider = entity.clone().into();
        assert_eq!(proto.name, entity.name);
        assert_eq!(proto.display_name, entity.display_name);
        assert_eq!(proto.api_key, entity.api_key);
        assert_eq!(proto.api_endpoint, entity.api_endpoint);

        // Proto -> Entity (round-trip)
        let entity_round: EntityProvider = proto.into();
        assert_eq!(entity_round, entity);
    }

    #[test]
    fn proto_to_entity_and_back_empty_fields() {
        let proto = ProtoProvider {
            name: "".to_string(),
            display_name: "".to_string(),
            api_key: "".to_string(),
            api_endpoint: "".to_string(),
        };

        // Proto -> Entity
        let entity: EntityProvider = proto.clone().into();
        assert_eq!(entity.name, proto.name);
        assert_eq!(entity.display_name, proto.display_name);
        assert_eq!(entity.api_key, proto.api_key);
        assert_eq!(entity.api_endpoint, proto.api_endpoint);

        // Entity -> Proto (round-trip)
        let proto_round: ProtoProvider = entity.clone().into();
        assert_eq!(proto_round, proto);

        let via_helper = provider_entity_to_proto(&entity);
        assert_eq!(via_helper, proto_round);

        let entity_via_helper = provider_proto_to_entity(&proto_round);
        assert_eq!(entity_via_helper, entity);

        let batch = provider_entities_to_proto(std::slice::from_ref(&entity_via_helper));
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0], proto_round);
    }
}
