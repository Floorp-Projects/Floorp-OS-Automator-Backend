// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

/*
-- plugin_package
CREATE TABLE plugin_package (
    package_id TEXT NOT NULL PRIMARY KEY,
    package_name TEXT,
    package_version TEXT,
    description TEXT,
    plugin_store_url TEXT,
    internal_plugin BOOLEAN NOT NULL DEFAULT FALSE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    deprecated BOOLEAN NOT NULL DEFAULT FALSE,
    installed_at TIMESTAMP,
    updated_at TIMESTAMP,
    provider_id TEXT
);

-- plugin_function
CREATE TABLE plugin_function (
    function_id TEXT NOT NULL,
    package_id TEXT NOT NULL,
    function_name TEXT,
    description TEXT,
    arguments TEXT,
    returns TEXT,
    function_define TEXT,
    version TEXT,
    PRIMARY KEY (package_id, function_id),
    FOREIGN KEY (package_id) REFERENCES plugin_package(package_id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX idx_plugin_function_function_id_unique ON plugin_function(function_id);

-- plugin_function_permission
CREATE TABLE plugin_function_permission (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    plugin_function_id TEXT NOT NULL,
    permission_id TEXT NOT NULL, -- Note: Defined as String in Rust, references Permission(Id) which is Integer
    FOREIGN KEY (plugin_function_id) REFERENCES plugin_function(function_id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permission(id) ON DELETE CASCADE
);

-- permission
CREATE TABLE permission (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    plugin_function_id TEXT NOT NULL,
    display_name TEXT,
    description TEXT,
    type INTEGER NOT NULL,
    resource_json TEXT,
    level INTEGER,
    FOREIGN KEY (plugin_function_id) REFERENCES plugin_function(function_id) ON DELETE CASCADE
);

-- provider
CREATE TABLE provider (
    name TEXT NOT NULL PRIMARY KEY,
    display_name TEXT,
    api_key TEXT,
    api_endpoint TEXT
);

-- model
CREATE TABLE model (
    name TEXT NOT NULL PRIMARY KEY,
    display_name TEXT,
    description TEXT,
    provider_name TEXT NOT NULL,
    FOREIGN KEY (provider_name) REFERENCES provider(name) ON DELETE CASCADE
);

-- workflow
CREATE TABLE workflow (
    id TEXT NOT NULL PRIMARY KEY,
    display_name TEXT,
    description TEXT,
    workflow_language INTEGER NOT NULL,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- workflow_code
CREATE TABLE workflow_code (
    id TEXT NOT NULL PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    code_revision INTEGER NOT NULL,
    code TEXT NOT NULL,
    language INTEGER NOT NULL,
    created_at TIMESTAMP,
    FOREIGN KEY (workflow_id) REFERENCES workflow(id) ON DELETE CASCADE
);

-- workflow_result
CREATE TABLE workflow_result (
    id TEXT NOT NULL PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    workflow_code_id TEXT NOT NULL,
    display_name TEXT,
    description TEXT,
    result TEXT,
    ran_at TIMESTAMP,
    result_type INTEGER NOT NULL,
    exit_code INTEGER,
    workflow_result_revision INTEGER NOT NULL,
    FOREIGN KEY (workflow_id) REFERENCES workflow(id) ON DELETE CASCADE,
    FOREIGN KEY (workflow_code_id) REFERENCES workflow_code(id) ON DELETE CASCADE
);

-- workflow_code_plugin_package
CREATE TABLE workflow_code_plugin_package (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    workflow_code_id TEXT NOT NULL,
    plugin_package_id TEXT NOT NULL,
    FOREIGN KEY (workflow_code_id) REFERENCES workflow_code(id) ON DELETE CASCADE,
    FOREIGN KEY (plugin_package_id) REFERENCES plugin_package(package_id) ON DELETE CASCADE
);

-- workflow_code_plugin_function
CREATE TABLE workflow_code_plugin_function (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    workflow_code_id TEXT NOT NULL,
    plugin_function_id TEXT NOT NULL,
    FOREIGN KEY (workflow_code_id) REFERENCES workflow_code(id) ON DELETE CASCADE,
    FOREIGN KEY (plugin_function_id) REFERENCES plugin_function(function_id) ON DELETE CASCADE
);

-- workflow_code_allowed_permission
CREATE TABLE workflow_code_allowed_permission (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    workflow_code_id TEXT NOT NULL,
    permission_id INTEGER NOT NULL,
    FOREIGN KEY (workflow_code_id) REFERENCES workflow_code(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permission(id) ON DELETE CASCADE
);

-- ext_plugin_package
-- Tracks externally installed plugin packages from the filesystem.
CREATE TABLE ext_plugin_package (
    plugin_package_id TEXT NOT NULL PRIMARY KEY,
    install_dir TEXT NOT NULL,
    missing BOOLEAN NOT NULL DEFAULT FALSE
);
*/
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PluginPackage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PluginPackage::PackageId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(PluginPackage::PackageName))
                    .col(string(PluginPackage::PackageVersion))
                    .col(ColumnDef::new(PluginPackage::Description).string().null())
                    .col(
                        ColumnDef::new(PluginPackage::PluginStoreUrl)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PluginPackage::InternalPlugin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PluginPackage::Verified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PluginPackage::Deprecated)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PluginPackage::InstalledAt)
                            .timestamp()
                            .null(),
                    )
                    .col(ColumnDef::new(PluginPackage::UpdatedAt).timestamp().null())
                    .col(ColumnDef::new(PluginPackage::ProviderId).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PluginFunction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PluginFunction::FunctionId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PluginFunction::PackageId)
                            .string()
                            .not_null(),
                    )
                    .col(string(PluginFunction::FunctionName))
                    .col(ColumnDef::new(PluginFunction::Description).string().null())
                    // Separate ID with colons (SQLite does not support for array)
                    // arguments and returns are reserved for backward compatibility
                    .col(ColumnDef::new(PluginFunction::Arguments).text().null())
                    .col(ColumnDef::new(PluginFunction::Returns).text().null())
                    // JSDoc-style function definition with parameters and return value (JSON)
                    .col(ColumnDef::new(PluginFunction::FunctionDefine).text().null())
                    // Version of the function (e.g., "1.0.0")
                    .col(ColumnDef::new(PluginFunction::Version).string().null())
                    .primary_key(
                        Index::create()
                            .col(PluginFunction::PackageId)
                            .col(PluginFunction::FunctionId),
                    )
                    .index(
                        Index::create()
                            .name("idx_plugin_function_function_id_unique")
                            .col(PluginFunction::FunctionId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plugin_function_package")
                            .from(PluginFunction::Table, PluginFunction::PackageId)
                            .to(PluginPackage::Table, PluginPackage::PackageId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PluginFunctionPermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PluginFunctionPermission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PluginFunctionPermission::PluginFunctionId)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plugin_function_permission_function")
                            .from(
                                PluginFunctionPermission::Table,
                                PluginFunctionPermission::PluginFunctionId,
                            )
                            .to(PluginFunction::Table, PluginFunction::FunctionId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(PluginFunctionPermission::PermissionId)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plugin_function_permission_permission")
                            .from(
                                PluginFunctionPermission::Table,
                                PluginFunctionPermission::PermissionId,
                            )
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Permission::PluginFunctionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Permission::DisplayName).string().null())
                    .col(ColumnDef::new(Permission::Description).string().null())
                    .col(ColumnDef::new(Permission::Type).integer().not_null())
                    .col(ColumnDef::new(Permission::ResourceJson).text().null())
                    .col(ColumnDef::new(Permission::Level).integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_permission_plugin_function")
                            .from(Permission::Table, Permission::PluginFunctionId)
                            .to(PluginFunction::Table, PluginFunction::FunctionId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Provider::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Provider::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(Provider::DisplayName))
                    .col(string(Provider::ApiKey))
                    .col(string(Provider::ApiEndpoint))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Model::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Model::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(Model::DisplayName))
                    .col(ColumnDef::new(Model::Description).string().null())
                    .col(ColumnDef::new(Model::ProviderName).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_model_provider")
                            .from(Model::Table, Model::ProviderName)
                            .to(Provider::Table, Provider::Name)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Workflow::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Workflow::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(Workflow::DisplayName))
                    .col(ColumnDef::new(Workflow::Description).string().null())
                    .col(
                        ColumnDef::new(Workflow::WorkflowLanguage)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Workflow::CreatedAt).timestamp().null())
                    .col(ColumnDef::new(Workflow::UpdatedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WorkflowCode::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkflowCode::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkflowCode::WorkflowId).string().not_null())
                    .col(
                        ColumnDef::new(WorkflowCode::CodeRevision)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkflowCode::Code).text().not_null())
                    .col(ColumnDef::new(WorkflowCode::Language).integer().not_null())
                    .col(ColumnDef::new(WorkflowCode::CreatedAt).timestamp().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_workflow")
                            .from(WorkflowCode::Table, WorkflowCode::WorkflowId)
                            .to(Workflow::Table, Workflow::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WorkflowResult::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkflowResult::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WorkflowResult::WorkflowId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkflowResult::WorkflowCodeId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkflowResult::DisplayName).string().null())
                    .col(ColumnDef::new(WorkflowResult::Description).string().null())
                    .col(ColumnDef::new(WorkflowResult::Result).text().null())
                    .col(ColumnDef::new(WorkflowResult::RanAt).timestamp().null())
                    .col(
                        ColumnDef::new(WorkflowResult::ResultType)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkflowResult::ExitCode).integer().null())
                    .col(
                        ColumnDef::new(WorkflowResult::WorkflowResultRevision)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_result_workflow")
                            .from(WorkflowResult::Table, WorkflowResult::WorkflowId)
                            .to(Workflow::Table, Workflow::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_result_code")
                            .from(WorkflowResult::Table, WorkflowResult::WorkflowCodeId)
                            .to(WorkflowCode::Table, WorkflowCode::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WorkflowCodePluginPackage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkflowCodePluginPackage::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WorkflowCodePluginPackage::WorkflowCodeId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkflowCodePluginPackage::PluginPackageId)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_plugin_package_code")
                            .from(
                                WorkflowCodePluginPackage::Table,
                                WorkflowCodePluginPackage::WorkflowCodeId,
                            )
                            .to(WorkflowCode::Table, WorkflowCode::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_plugin_package_package")
                            .from(
                                WorkflowCodePluginPackage::Table,
                                WorkflowCodePluginPackage::PluginPackageId,
                            )
                            .to(PluginPackage::Table, PluginPackage::PackageId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WorkflowCodePluginFunction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkflowCodePluginFunction::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WorkflowCodePluginFunction::WorkflowCodeId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkflowCodePluginFunction::PluginFunctionId)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_plugin_function_code")
                            .from(
                                WorkflowCodePluginFunction::Table,
                                WorkflowCodePluginFunction::WorkflowCodeId,
                            )
                            .to(WorkflowCode::Table, WorkflowCode::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_plugin_function_function")
                            .from(
                                WorkflowCodePluginFunction::Table,
                                WorkflowCodePluginFunction::PluginFunctionId,
                            )
                            .to(PluginFunction::Table, PluginFunction::FunctionId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WorkflowCodeAllowedPermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkflowCodeAllowedPermission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WorkflowCodeAllowedPermission::WorkflowCodeId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkflowCodeAllowedPermission::PermissionId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_allowed_permission_code")
                            .from(
                                WorkflowCodeAllowedPermission::Table,
                                WorkflowCodeAllowedPermission::WorkflowCodeId,
                            )
                            .to(WorkflowCode::Table, WorkflowCode::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workflow_code_allowed_permission_permission")
                            .from(
                                WorkflowCodeAllowedPermission::Table,
                                WorkflowCodeAllowedPermission::PermissionId,
                            )
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // External Plugin Package table
        manager
            .create_table(
                Table::create()
                    .table(ExtPluginPackage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExtPluginPackage::PluginPackageId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExtPluginPackage::InstallDir)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExtPluginPackage::Missing)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExtPluginPackage::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(WorkflowCodeAllowedPermission::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(WorkflowCodePluginFunction::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(WorkflowCodePluginPackage::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(WorkflowResult::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(WorkflowCode::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Workflow::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Model::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(PluginFunctionPermission::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(PluginFunction::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PluginPackage::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Provider::Table).to_owned())
            .await
    }
}

// ------ AI Related Tables -------
#[derive(DeriveIden)]
enum Provider {
    Table,
    Name,
    DisplayName,
    ApiKey,
    ApiEndpoint,
}

#[derive(DeriveIden)]
enum Model {
    Table,
    Name,
    DisplayName,
    Description,
    ProviderName,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum Workflow {
    Table,
    Id,
    DisplayName,
    Description,
    WorkflowLanguage,
    CreatedAt,
    UpdatedAt,
}

// ------ Plugin Related Tables -------

#[derive(DeriveIden)]
enum PluginPackage {
    Table,
    PackageId,
    PackageName,
    PackageVersion,
    Description,
    PluginStoreUrl,
    InternalPlugin,
    Verified,
    Deprecated,
    InstalledAt,
    UpdatedAt,
    ProviderId,
}

#[derive(DeriveIden)]
enum PluginFunction {
    Table,
    PackageId,
    FunctionId,
    FunctionName,
    Description,
    Arguments,
    Returns,
    FunctionDefine,
    Version,
}

#[derive(DeriveIden)]
enum PluginFunctionPermission {
    Table,
    Id,
    PluginFunctionId,
    PermissionId,
}

// Permission Related Table
#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    PluginFunctionId,
    DisplayName,
    Description,
    Type,
    ResourceJson,
    Level,
}

// Workflow Code Related Tables
#[derive(DeriveIden)]
enum WorkflowCode {
    Table,
    Id,
    WorkflowId,
    CodeRevision,
    Code,
    Language,
    CreatedAt,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum WorkflowResult {
    Table,
    Id,
    WorkflowId,
    WorkflowCodeId,
    DisplayName,
    Description,
    Result,
    RanAt,
    ResultType,
    ExitCode,
    WorkflowResultRevision,
}

#[derive(DeriveIden)]
enum WorkflowCodePluginPackage {
    Table,
    Id,
    WorkflowCodeId,
    PluginPackageId,
}

#[derive(DeriveIden)]
enum WorkflowCodePluginFunction {
    Table,
    Id,
    WorkflowCodeId,
    PluginFunctionId,
}

#[derive(DeriveIden)]
enum WorkflowCodeAllowedPermission {
    Table,
    Id,
    WorkflowCodeId,
    PermissionId,
}

// ------ External Plugin Related Tables -------
#[derive(DeriveIden)]
enum ExtPluginPackage {
    Table,
    PluginPackageId,
    InstallDir,
    Missing,
}
