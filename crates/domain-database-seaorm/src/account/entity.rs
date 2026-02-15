use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum AccountTypeEnum {
    #[sea_orm(string_value = "Asset")]
    Asset,
    #[sea_orm(string_value = "Liability")]
    Liability,
    #[sea_orm(string_value = "Equity")]
    Equity,
    #[sea_orm(string_value = "Income")]
    Income,
    #[sea_orm(string_value = "Expense")]
    Expense,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub version: i32,
    pub created_at: Option<DateTimeUtc>,
    pub last_modified_at: Option<DateTimeUtc>,
    pub archived_at: Option<DateTimeUtc>,
    pub journal_id: String,
    pub parent_id: Option<String>,
    pub r#type: AccountTypeEnum,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::journal::entity::Entity",
        from = "Column::JournalId",
        to = "crate::journal::entity::Column::Id"
    )]
    Journal,
    #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    Parent,
    #[sea_orm(has_many = "super::tag_entity::Entity")]
    Tags,
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

impl ActiveModelBehavior for ActiveModel {}
