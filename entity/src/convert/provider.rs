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

use sapphillon_core::proto::sapphillon::ai::v1::Provider as ProtoProvider;
use crate::entity::provider::Model as EntityProvider;

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