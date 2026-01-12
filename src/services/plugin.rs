// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later
// Sapphillon
//
//
//
//

use std::sync::Arc;

use database::plugin::list_plugins;
use log::{debug, error};
use sapphillon_core::proto::google::rpc::{Code as RpcCode, Status as RpcStatus};
use sapphillon_core::proto::sapphillon::v1::plugin_service_server::PluginService;
use sapphillon_core::proto::sapphillon::v1::{
    InstallPluginRequest, InstallPluginResponse, ListPluginsRequest, ListPluginsResponse,
    UninstallPluginRequest, UninstallPluginResponse,
};
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

        let (plugins, next_token) = list_plugins(&self.db, next_page_token, page_size)
            .await
            .map_err(Self::map_db_error)?;

        let response = ListPluginsResponse {
            plugins,
            next_page_token: next_token,
            status: Self::ok_status("plugins listed"),
        };

        Ok(Response::new(response))
    }

    async fn install_plugin(
        &self,
        request: Request<InstallPluginRequest>,
    ) -> Result<Response<InstallPluginResponse>, Status> {
        use crate::plugin_installer::{InstallError, install_plugin_from_uri};
        use sapphillon_core::proto::google::rpc::Code as RpcCode;

        let req = request.into_inner();
        debug!("install_plugin request received: uri='{}'", req.uri);

        // Get save directory from global state
        let save_dir = crate::GLOBAL_STATE.get_ext_plugin_save_dir().await;

        // Install the plugin using the installer module
        match install_plugin_from_uri(&self.db, &save_dir, &req.uri).await {
            Ok(result) => {
                debug!(
                    "plugin installed successfully: {}",
                    result.plugin_package_id
                );
                Ok(Response::new(InstallPluginResponse {
                    plugin: None, // Plugin metadata not available from raw download
                    status: Self::ok_status(format!(
                        "plugin installed: {}",
                        result.plugin_package_id
                    )),
                }))
            }
            Err(e) => {
                error!("failed to install plugin: {e}");
                let code = match &e {
                    InstallError::EmptyUri
                    | InstallError::UnsupportedScheme(_)
                    | InstallError::InvalidUriFormat(_) => RpcCode::InvalidArgument,
                    InstallError::DownloadFailed(_) | InstallError::FileReadFailed(_) => {
                        RpcCode::Unavailable
                    }
                    InstallError::AlreadyInstalled(_) => RpcCode::AlreadyExists,
                    InstallError::InstallFailed(_) => RpcCode::Internal,
                };
                Ok(Response::new(InstallPluginResponse {
                    plugin: None,
                    status: Some(RpcStatus {
                        code: code as i32,
                        message: e.to_string(),
                        details: vec![],
                    }),
                }))
            }
        }
    }

    async fn uninstall_plugin(
        &self,
        request: Request<UninstallPluginRequest>,
    ) -> Result<Response<UninstallPluginResponse>, Status> {
        use sapphillon_core::proto::google::rpc::Code as RpcCode;

        let req = request.into_inner();
        debug!(
            "uninstall_plugin request received: package_id='{}'",
            req.package_id
        );

        // Validate package_id is not empty
        if req.package_id.trim().is_empty() {
            return Ok(Response::new(UninstallPluginResponse {
                status: Some(RpcStatus {
                    code: RpcCode::InvalidArgument as i32,
                    message: "package_id field is required".to_string(),
                    details: vec![],
                }),
            }));
        }

        // Uninstall the plugin
        match crate::ext_plugin_manager::uninstall_ext_plugin(&self.db, &req.package_id).await {
            Ok(()) => {
                debug!("plugin uninstalled successfully: {}", req.package_id);
                Ok(Response::new(UninstallPluginResponse {
                    status: Self::ok_status(format!("plugin uninstalled: {}", req.package_id)),
                }))
            }
            Err(e) => {
                error!("failed to uninstall plugin: {e}");
                let code = if e.to_string().contains("not found") {
                    RpcCode::NotFound
                } else {
                    RpcCode::Internal
                };
                Ok(Response::new(UninstallPluginResponse {
                    status: Some(RpcStatus {
                        code: code as i32,
                        message: e.to_string(),
                        details: vec![],
                    }),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, DbBackend, Statement};
    use tempfile::TempDir;

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;

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

        // ext_plugin_package table
        let sql_ext = r#"
			CREATE TABLE ext_plugin_package (
				plugin_package_id TEXT NOT NULL PRIMARY KEY,
				install_dir TEXT NOT NULL,
				missing INTEGER NOT NULL DEFAULT 0
			)
		"#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_ext.to_string(),
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

        let resp = service
            .list_plugins(req)
            .await
            .expect("list_plugins failed");
        let inner = resp.into_inner();
        assert!(inner.plugins.is_empty());
        assert!(inner.next_page_token.is_empty());
    }

    #[tokio::test]
    async fn test_install_plugin_empty_uri() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        let req = Request::new(InstallPluginRequest {
            uri: "".to_string(),
        });

        let resp = service
            .install_plugin(req)
            .await
            .expect("install_plugin should not fail");
        let inner = resp.into_inner();

        // Should return InvalidArgument error
        assert!(inner.status.is_some());
        let status = inner.status.unwrap();
        assert_eq!(
            status.code,
            sapphillon_core::proto::google::rpc::Code::InvalidArgument as i32
        );
        assert!(status.message.contains("required"));
    }

    #[tokio::test]
    async fn test_install_plugin_unsupported_scheme() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        let req = Request::new(InstallPluginRequest {
            uri: "ftp://example.com/author/pkg/1.0.0/package.js".to_string(),
        });

        let resp = service
            .install_plugin(req)
            .await
            .expect("install_plugin should not fail");
        let inner = resp.into_inner();

        // Should return InvalidArgument error
        assert!(inner.status.is_some());
        let status = inner.status.unwrap();
        assert_eq!(
            status.code,
            sapphillon_core::proto::google::rpc::Code::InvalidArgument as i32
        );
        assert!(status.message.contains("unsupported"));
    }

    #[tokio::test]
    async fn test_install_plugin_from_file() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        // Create a temporary directory with plugin structure
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let plugin_dir = temp_dir.path().join("test-author/test-pkg/1.0.0");
        std::fs::create_dir_all(&plugin_dir).expect("failed to create plugin dir");

        let plugin_file = plugin_dir.join("package.js");
        std::fs::write(&plugin_file, b"console.log('test plugin');")
            .expect("failed to write plugin");

        // Set ext_plugin_save_dir in global state
        crate::GLOBAL_STATE
            .async_set_ext_plugin_save_dir(Some(temp_dir.path().to_string_lossy().to_string()))
            .await;

        let file_uri = format!("file://{}", plugin_file.to_string_lossy());
        let req = Request::new(InstallPluginRequest { uri: file_uri });

        let resp = service
            .install_plugin(req)
            .await
            .expect("install_plugin should not fail");
        let inner = resp.into_inner();

        // Should return OK
        assert!(inner.status.is_some());
        let status = inner.status.unwrap();
        assert_eq!(
            status.code,
            sapphillon_core::proto::google::rpc::Code::Ok as i32
        );
        assert!(status.message.contains("installed"));
    }

    #[tokio::test]
    async fn test_uninstall_plugin_empty_package_id() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        let req = Request::new(UninstallPluginRequest {
            package_id: "".to_string(),
        });

        let resp = service
            .uninstall_plugin(req)
            .await
            .expect("uninstall_plugin should not fail");
        let inner = resp.into_inner();

        // Should return InvalidArgument error
        assert!(inner.status.is_some());
        let status = inner.status.unwrap();
        assert_eq!(
            status.code,
            sapphillon_core::proto::google::rpc::Code::InvalidArgument as i32
        );
        assert!(status.message.contains("required"));
    }

    #[tokio::test]
    async fn test_uninstall_plugin_not_found() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        let req = Request::new(UninstallPluginRequest {
            package_id: "nonexistent/plugin/1.0.0".to_string(),
        });

        let resp = service
            .uninstall_plugin(req)
            .await
            .expect("uninstall_plugin should not fail");
        let inner = resp.into_inner();

        // Should return NotFound error
        assert!(inner.status.is_some());
        let status = inner.status.unwrap();
        assert_eq!(
            status.code,
            sapphillon_core::proto::google::rpc::Code::NotFound as i32
        );
        assert!(status.message.contains("not found"));
    }

    #[tokio::test]
    async fn test_install_and_uninstall_plugin() {
        let db = setup_db().await.expect("db setup failed");
        let service = MyPluginService::new(db);

        // Create a temporary directory for plugins
        let save_dir = TempDir::new().expect("failed to create save dir");
        let source_dir = TempDir::new().expect("failed to create source dir");

        // Create plugin source file
        let plugin_source_dir = source_dir.path().join("myauthor/mypkg/2.0.0");
        std::fs::create_dir_all(&plugin_source_dir).expect("failed to create source dir");
        let plugin_file = plugin_source_dir.join("package.js");
        std::fs::write(&plugin_file, b"console.log('my plugin');").expect("failed to write plugin");

        // Set save directory
        crate::GLOBAL_STATE
            .async_set_ext_plugin_save_dir(Some(save_dir.path().to_string_lossy().to_string()))
            .await;

        // Install the plugin
        let file_uri = format!("file://{}", plugin_file.to_string_lossy());
        let install_req = Request::new(InstallPluginRequest {
            uri: file_uri.clone(),
        });

        let install_resp = service
            .install_plugin(install_req)
            .await
            .expect("install_plugin should not fail");
        let install_inner = install_resp.into_inner();

        assert!(install_inner.status.is_some());
        let install_status = install_inner.status.unwrap();
        assert_eq!(
            install_status.code,
            sapphillon_core::proto::google::rpc::Code::Ok as i32
        );

        // Verify plugin file was created in save directory
        let installed_path = save_dir.path().join("myauthor/mypkg/2.0.0/package.js");
        assert!(installed_path.exists());

        // Uninstall the plugin
        let uninstall_req = Request::new(UninstallPluginRequest {
            package_id: "myauthor/mypkg/2.0.0".to_string(),
        });

        let uninstall_resp = service
            .uninstall_plugin(uninstall_req)
            .await
            .expect("uninstall_plugin should not fail");
        let uninstall_inner = uninstall_resp.into_inner();

        assert!(uninstall_inner.status.is_some());
        let uninstall_status = uninstall_inner.status.unwrap();
        assert_eq!(
            uninstall_status.code,
            sapphillon_core::proto::google::rpc::Code::Ok as i32
        );

        // Verify plugin file was removed
        assert!(!installed_path.exists());
    }
}
