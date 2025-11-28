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

use std::sync::Arc;

use database::plugin::list_plugins;
use log::{debug, error};
use sapphillon_core::proto::google::rpc::{Code as RpcCode, Status as RpcStatus};
use sapphillon_core::proto::sapphillon::v1::plugin_service_server::PluginService;
use sapphillon_core::proto::sapphillon::v1::{ListPluginsRequest, ListPluginsResponse};
use sea_orm::{DatabaseConnection, DbErr};
use tonic::{Request, Response, Status};

#[derive(Clone, Debug)]
pub struct MyPluginService {
    db: Arc<DatabaseConnection>,
}

impl MyPluginService {
    /// Creates a new plugin service backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    fn ok_status(message: impl Into<String>) -> Option<RpcStatus> {
        Some(RpcStatus {
            code: RpcCode::Ok as i32,
            message: message.into(),
            details: vec![],
        })
    }

    fn map_db_error(err: DbErr) -> Status {
        error!("database operation failed: {err:?}");
        Status::internal("database operation failed")
    }
}

#[tonic::async_trait]
impl PluginService for MyPluginService {
    async fn list_plugins(
        &self,
        request: Request<ListPluginsRequest>,
    ) -> Result<Response<ListPluginsResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "list_plugins request received: page_size={page_size}, page_token='{page_token}'",
            page_size = req.page_size,
            page_token = req.page_token.as_str()
        );

        let page_size = if req.page_size <= 0 {
            None
        } else {
            Some(req.page_size as u32)
        };

        let next_page_token = if req.page_token.trim().is_empty() {
            None
        } else {
            Some(req.page_token)
        };

        let (plugins, next_token) =
            list_plugins(&self.db, next_page_token, page_size)
                .await
                .map_err(Self::map_db_error)?;

        let response = ListPluginsResponse {
            plugins,
            next_page_token: next_token,
            status: Self::ok_status("plugins listed"),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DbBackend, ConnectionTrait, Statement};

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        // plugin_package table
        let sql_pkg = r#"
			CREATE TABLE plugin_package (
				package_id TEXT PRIMARY KEY,
				package_name TEXT NOT NULL,
				package_version TEXT NOT NULL,
				description TEXT,
				plugin_store_url TEXT,
				internal_plugin INTEGER NOT NULL,
				verified INTEGER NOT NULL,
				deprecated INTEGER NOT NULL,
				installed_at TEXT,
				updated_at TEXT
			)
		"#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pkg.to_string(),
        ))
        .await?;

        // plugin_function table
        let sql_pf = r#"
			CREATE TABLE plugin_function (
				function_id TEXT NOT NULL UNIQUE,
				package_id TEXT NOT NULL,
				function_name TEXT NOT NULL,
				description TEXT,
				arguments TEXT,
				returns TEXT,
				PRIMARY KEY (function_id, package_id)
			)
		"#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pf.to_string(),
        ))
        .await?;

        // permission table
        let sql_perm = r#"
			CREATE TABLE permission (
				id INTEGER PRIMARY KEY,
				plugin_function_id TEXT NOT NULL,
				display_name TEXT,
				description TEXT,
				"type" INTEGER NOT NULL,
				resource_json TEXT,
				level INTEGER
			)
		"#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_perm.to_string(),
        ))
        .await?;

        // plugin_function_permission table
        let sql_pfp = r#"
			CREATE TABLE plugin_function_permission (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				plugin_function_id TEXT NOT NULL,
				permission_id TEXT NOT NULL
			)
		"#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pfp.to_string(),
        ))
        .await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_list_plugins_empty() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        let req = Request::new(ListPluginsRequest {
            page_size: 10,
            page_token: "".to_string(),
        });

        let resp = service.list_plugins(req).await.expect("list_plugins failed");
        let inner = resp.into_inner();
        assert!(inner.plugins.is_empty());
        assert!(inner.next_page_token.is_empty());
    }
}
