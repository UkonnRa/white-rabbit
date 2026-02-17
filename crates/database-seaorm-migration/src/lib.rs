pub use sea_orm_migration::prelude::*;

mod m20260209_000001_create_journal_table;
mod m20260209_000002_create_journal_tag_table;
mod m20260209_000003_create_account_table;
mod m20260209_000004_create_account_tag_table;
mod m20260209_000005_create_record_table;
mod m20260209_000006_create_record_tag_table;
mod m20260209_000007_create_record_item_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260209_000001_create_journal_table::Migration),
            Box::new(m20260209_000002_create_journal_tag_table::Migration),
            Box::new(m20260209_000003_create_account_table::Migration),
            Box::new(m20260209_000004_create_account_tag_table::Migration),
            Box::new(m20260209_000005_create_record_table::Migration),
            Box::new(m20260209_000006_create_record_tag_table::Migration),
            Box::new(m20260209_000007_create_record_item_table::Migration),
        ]
    }
}
