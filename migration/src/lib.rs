pub use sea_orm_migration::prelude::*;

mod m20250908_000001_create_providers_and_models;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250908_000001_create_providers_and_models::Migration),
        ]
    }
}
