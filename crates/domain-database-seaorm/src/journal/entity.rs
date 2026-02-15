use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "journal")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub version: i32,
    pub created_at: Option<DateTimeUtc>,
    pub last_modified_at: Option<DateTimeUtc>,
    pub archived_at: Option<DateTimeUtc>,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tag_entity::Entity")]
    Tags,
}

impl Related<super::tag_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
