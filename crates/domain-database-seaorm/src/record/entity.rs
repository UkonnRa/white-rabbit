use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum RecordKindEnum {
    #[sea_orm(string_value = "Transaction")]
    Transaction,
    #[sea_orm(string_value = "Validation")]
    Validation,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub version: i32,
    pub created_at: Option<DateTimeUtc>,
    pub last_modified_at: Option<DateTimeUtc>,
    pub journal_id: String,
    pub date: Date,
    pub kind: RecordKindEnum,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    #[sea_orm(column_type = "Text")]
    pub payee: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::journal::entity::Entity",
        from = "Column::JournalId",
        to = "crate::journal::entity::Column::Id"
    )]
    Journal,
    #[sea_orm(has_many = "super::tag_entity::Entity")]
    Tags,
    #[sea_orm(has_many = "super::item_entity::Entity")]
    Items,
}

impl Related<crate::journal::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Journal.def()
    }
}

impl Related<super::tag_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }
}

impl Related<super::item_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
