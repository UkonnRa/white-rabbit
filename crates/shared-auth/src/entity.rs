use shared::{Entity, EntityId};

pub trait AuthEntity: Entity {
    type OperatorId: EntityId;

    fn created_by_id(&self) -> Option<&Self::OperatorId>;
    fn last_modified_by_id(&self) -> Option<&Self::OperatorId>;
}
