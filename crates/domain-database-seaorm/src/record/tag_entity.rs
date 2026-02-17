use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub record_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag: String,
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
