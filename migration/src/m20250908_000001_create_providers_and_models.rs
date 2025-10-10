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
                    .col(ColumnDef::new(PluginFunction::PermissionId).text().null())
                    .col(ColumnDef::new(PluginFunction::Arguments).text().null())
                    .col(ColumnDef::new(PluginFunction::Returns).text().null())
                    .primary_key(
                        Index::create()
                            .col(PluginFunction::PackageId)
                            .col(PluginFunction::FunctionId),
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
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permission::Id)
                            .string()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Permission::DisplayName).string().null())
                    .col(ColumnDef::new(Permission::Description).string().null())
                    .col(ColumnDef::new(Permission::Type).integer().not_null())
                    .col(ColumnDef::new(Permission::ResourceJson).text().null())
                    .col(ColumnDef::new(Permission::Level).integer().null())
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Model::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PluginFunction::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PluginPackage::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Provider::Table).to_owned())
            .await
    }
}

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
}

#[derive(DeriveIden)]
enum PluginFunction {
    Table,
    PackageId,
    FunctionId,
    FunctionName,
    Description,
    PermissionId,
    Arguments,
    Returns,
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    DisplayName,
    Description,
    Type,
    ResourceJson,
    Level,
}
