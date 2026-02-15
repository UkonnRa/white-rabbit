use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Journal::Table)
                    .if_not_exists()
                    .col(string(Journal::Id).primary_key())
                    .col(integer(Journal::Version).not_null().default(0))
                    .col(timestamp_with_time_zone_null(Journal::CreatedAt))
                    .col(timestamp_with_time_zone_null(Journal::LastModifiedAt))
                    .col(timestamp_with_time_zone_null(Journal::ArchivedAt))
                    .col(string(Journal::Name).not_null().unique_key())
                    .col(text(Journal::Description).not_null().default(""))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Journal::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Journal {
    Table,
    Id,
    Version,
    CreatedAt,
    LastModifiedAt,
    ArchivedAt,
    Name,
    Description,
}
