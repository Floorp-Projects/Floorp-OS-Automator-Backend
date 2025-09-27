use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
