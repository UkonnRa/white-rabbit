use crate::repository::{InMemoryReadRepository, InMemoryWriteRepository};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use shared::{DomainModel, EntityId, Id, Persistence, Result};
use shared::{ReadRepository, Specification, WriteRepository};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, DomainModel)]
pub struct User {
    id: UserId,
    version: usize,
    created_at: Option<DateTime<Utc>>,
    created_by_id: Option<UserId>,
    last_modified_at: Option<DateTime<Utc>>,
    last_modified_by_id: Option<UserId>,
    name: String,
    manager_id: Option<UserId>, // ID-only reference (SDR-aligned)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct UserPO {
    id: UserId,
    version: usize,
    created_at: Option<DateTime<Utc>>,
    created_by_id: Option<UserId>,
    last_modified_at: Option<DateTime<Utc>>,
    last_modified_by_id: Option<UserId>,
    name: String,
    manager_id: Option<UserId>,
}

impl Persistence for UserPO {
    type Id = UserId;
    type OperatorId = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> usize {
        self.version
    }

    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn created_by_id(&self) -> Option<&Self::OperatorId> {
        self.created_by_id.as_ref()
    }

    fn last_modified_at(&self) -> Option<DateTime<Utc>> {
        self.last_modified_at
    }

    fn last_modified_by_id(&self) -> Option<&Self::OperatorId> {
        self.last_modified_by_id.as_ref()
    }
}

#[derive(Debug)]
enum UserSpecification {
    Id(HashSet<UserId>),
    Name(HashSet<String>),
    Manager(Option<HashSet<UserId>>),
}

impl Specification for UserSpecification {}

#[derive(Debug, Default)]
struct UserRepository(HashMap<String, UserPO>);

#[async_trait::async_trait]
impl InMemoryReadRepository<UserSpecification> for UserRepository {
    type Persistence = UserPO;

    fn satisfies(
        &self,
        persistence: &Self::Persistence,
        specification: &UserSpecification,
    ) -> bool {
        match specification {
            UserSpecification::Id(value) => value.contains(&persistence.id),
            UserSpecification::Name(value) => value
                .iter()
                .any(|v| v.trim().to_lowercase() == persistence.name.trim().to_lowercase()),
            UserSpecification::Manager(Some(value)) => {
                if let Some(manager_id) = &persistence.manager_id {
                    value.contains(manager_id)
                } else {
                    false
                }
            }
            UserSpecification::Manager(None) => persistence.manager_id.is_none(),
        }
    }

    fn get_storage(&self) -> &HashMap<String, Self::Persistence> {
        &self.0
    }

    fn get_storage_mut(&mut self) -> &mut HashMap<String, Self::Persistence> {
        &mut self.0
    }

    fn convert_to_entity(&self, persistence: Self::Persistence) -> Self::Entity {
        User {
            id: persistence.id,
            version: persistence.version,
            created_at: persistence.created_at,
            created_by_id: persistence.created_by_id,
            last_modified_at: persistence.last_modified_at,
            last_modified_by_id: persistence.last_modified_by_id,
            name: persistence.name,
            manager_id: persistence.manager_id,
        }
    }

    fn convert_to_persistence(&self, entity: &Self::Entity) -> Self::Persistence {
        UserPO {
            id: entity.id.clone(),
            version: entity.version,
            created_at: entity.created_at,
            created_by_id: entity.created_by_id.clone(),
            last_modified_at: entity.last_modified_at,
            last_modified_by_id: entity.last_modified_by_id.clone(),
            name: entity.name.clone(),
            manager_id: entity.manager_id.clone(),
        }
    }
}

#[async_trait::async_trait]
impl InMemoryWriteRepository<UserSpecification> for UserRepository {}

#[async_trait::async_trait]
impl WriteRepository<UserSpecification> for UserRepository {
    async fn save_all(
        &mut self,
        entities: &[Self::Entity],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        self.__save_all(entities).await
    }

    async fn delete_all_by_ids(
        &mut self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        self.__delete_all_by_ids(ids).await
    }
}

#[async_trait::async_trait]
impl ReadRepository<UserSpecification> for UserRepository {
    type Entity = User;

    async fn find_all_by_ids(
        &self,
        ids: &[Id<Self::Entity>],
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        self.__find_all_by_ids(ids).await
    }

    async fn find_all(
        &self,
        spec: &UserSpecification,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Self::Entity>, Self::Entity>> {
        self.__find_all(spec, limit).await
    }
}

#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let user_alice_id = UserId::from_value("1");
    let user_bob_id = UserId::from_value("2");
    let user_charlie_id = UserId::from_value("3");
    let user_betty_id = UserId::from_value("4");

    let users = Arc::new(vec![
        User {
            id: UserId::from_value("1"),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 1, 1, 0, 0, 0).unwrap()),
            created_by_id: None,
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 2, 1, 0, 0, 0).unwrap()),
            last_modified_by_id: None,
            name: "Alice".to_string(),
            manager_id: None,
        },
        User {
            id: UserId::from_value("2"),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 3, 1, 0, 0, 0).unwrap()),
            created_by_id: None,
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 4, 1, 0, 0, 0).unwrap()),
            last_modified_by_id: None,
            name: "Bob".to_string(),
            manager_id: Some(user_alice_id.clone()),
        },
        User {
            id: UserId::from_value("3"),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 5, 1, 0, 0, 0).unwrap()),
            created_by_id: None,
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 6, 1, 0, 0, 0).unwrap()),
            last_modified_by_id: None,
            name: "Charlie".to_string(),
            manager_id: Some(user_bob_id.clone()),
        },
        User {
            id: UserId::from_value("4"),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 7, 1, 0, 0, 0).unwrap()),
            created_by_id: None,
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 8, 1, 0, 0, 0).unwrap()),
            last_modified_by_id: None,
            name: "Betty".to_string(),
            manager_id: Some(user_alice_id.clone()),
        },
    ]);

    let _user_alice = &users[0];
    let user_bob = &users[1];
    let user_charlie = &users[2];
    let _user_betty = &users[3];

    let user_repo = Mutex::new(UserRepository::default());

    let mut user_repo = user_repo.lock().await;
    user_repo.save_all(&users).await?;

    let result = user_repo
        .find_all_by_ids(&[user_charlie_id.clone(), user_betty_id.clone()])
        .await?;
    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&user_charlie_id));
    assert!(result.contains_key(&user_betty_id));

    let result2 = user_repo
        .find_all(
            &UserSpecification::Id(HashSet::from_iter([
                user_charlie_id.clone(),
                user_betty_id.clone(),
            ])),
            None,
        )
        .await?;
    assert_eq!(result2.len(), 2);
    assert_eq!(result2, result);

    let new_charlie = result
        .values()
        .find(|u| u.name == user_charlie.name)
        .unwrap();
    assert_eq!(new_charlie.manager_id, user_charlie.manager_id);

    let result = user_repo
        .find_all(
            &UserSpecification::Manager(Some(HashSet::from_iter([user_alice_id.clone()]))),
            None,
        )
        .await?;
    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&user_betty_id));
    assert!(result.contains_key(&user_bob_id));

    let result = user_repo
        .find_all(
            &UserSpecification::Name(HashSet::from_iter([
                user_charlie.name.clone(),
                user_bob.name.to_lowercase(),
                "Nonexistent".to_string(),
            ])),
            None,
        )
        .await?;
    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&user_charlie_id));
    assert!(result.contains_key(&user_bob_id));

    let results = user_repo.delete_all_by_ids(&[user_bob_id.clone()]).await?;
    assert_eq!(results.len(), 1);
    assert!(results.contains_key(&user_bob_id));

    let result = user_repo
        .find_all_by_ids(&[user_charlie_id.clone()])
        .await?;
    assert_eq!(result.len(), 1);
    // Manager ID still references the deleted user (referential integrity is a separate concern)
    assert_eq!(
        result.values().next().unwrap().manager_id,
        Some(user_bob_id.clone())
    );

    Ok(())
}
