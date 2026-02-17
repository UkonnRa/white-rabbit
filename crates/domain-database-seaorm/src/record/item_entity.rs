use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub record_id: String,
    pub account_id: String,
    pub account_type: Option<String>,
    pub amount_number: String,
    pub amount_unit: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub price_number: Option<String>,
    pub price_unit: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub costs: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::RecordId",
        to = "super::entity::Column::Id"
    )]
    Record,
}

impl Related<super::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Record.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
