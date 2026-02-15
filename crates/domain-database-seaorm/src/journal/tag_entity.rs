use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "journal_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub journal_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::JournalId",
        to = "super::entity::Column::Id"
    )]
    Journal,
}

impl Related<super::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Journal.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
