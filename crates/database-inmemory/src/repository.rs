use shared::entity::Entity;
use shared::persistence::Persistence;
use shared::repository::ReadRepository;
use std::collections::{HashMap, HashSet};

#[async_trait::async_trait]
pub trait InMemoryReadRepository: ReadRepository {
    type Persistence: Persistence;

    fn satisfies(
        &self,
        persistence: &Self::Persistence,
        specification: &Self::Specification,
    ) -> bool;

    fn get_storage(&self) -> &HashMap<String, Self::Persistence>;

    fn get_storage_mut(&mut self) -> &mut HashMap<String, Self::Persistence>;

    fn convert_to_entity(&self, persistence: Self::Persistence) -> Self::Entity;

    fn convert_to_persistence(&self, entity: &Self::Entity) -> Self::Persistence;

    async fn __find_all_by_ids(&self, ids: &[&str]) -> HashSet<Self::Entity> {
        let storage = self.get_storage();
        ids.iter()
            .filter_map(|id| storage.get(*id).cloned())
            .map(|po| self.convert_to_entity(po))
            .collect()
    }

    async fn __find_all(
        &self,
        spec: &Self::Specification,
        limit: Option<usize>,
    ) -> HashSet<Self::Entity> {
        self.get_storage()
            .values()
            .filter_map(|po| {
                if self.satisfies(po, spec) {
                    Some(self.convert_to_entity(po.clone()))
                } else {
                    None
                }
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect()
    }
}

#[async_trait::async_trait]
pub trait InMemoryWriteRepository: InMemoryReadRepository {
    async fn __save_all(&mut self, entities: &[&Self::Entity]) -> HashSet<Self::Entity> {
        let mut saved_pos = HashSet::new();
        for entity in entities {
            let po = self.convert_to_persistence(entity);
            if let Some(po) = self
                .get_storage_mut()
                .insert(po.id().to_string(), po.clone())
            {
                saved_pos.insert(po);
            }
        }
        saved_pos
            .into_iter()
            .map(|po| self.convert_to_entity(po))
            .collect()
    }

    async fn __delete_all_by_ids(&mut self, ids: &[&str]) -> HashSet<Self::Entity> {
        let mut deleted_pos = HashSet::new();
        for id in ids {
            let storage = self.get_storage_mut();
            if let Some(po) = storage.remove(*id) {
                deleted_pos.insert(po);
            }
        }
        deleted_pos
            .into_iter()
            .map(|po| self.convert_to_entity(po))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::{InMemoryReadRepository, InMemoryWriteRepository};
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Serialize};
    use shared::entity::Entity;
    use shared::persistence::Persistence;
    use shared::repository::{ReadRepository, WriteRepository};
    use shared::specification::Specification;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct User {
        id: String,
        version: usize,
        created_at: Option<DateTime<Utc>>,
        last_modified_at: Option<DateTime<Utc>>,
        name: String,
        manager: Option<Arc<User>>,
    }

    impl Entity for User {
        type Operator = User;

        fn id(&self) -> &str {
            self.id.as_str()
        }

        fn version(&self) -> usize {
            self.version
        }

        fn created_at(&self) -> Option<DateTime<Utc>> {
            self.created_at
        }

        fn created_by(&self) -> Option<&Self::Operator> {
            None
        }

        fn last_modified_at(&self) -> Option<DateTime<Utc>> {
            self.last_modified_at
        }

        fn last_modified_by(&self) -> Option<&Self::Operator> {
            None
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    struct UserPO {
        id: String,
        version: usize,
        created_at: Option<DateTime<Utc>>,
        last_modified_at: Option<DateTime<Utc>>,
        name: String,
        manager_id: Option<String>,
    }

    impl Persistence for UserPO {
        fn id(&self) -> &str {
            self.id.as_str()
        }

        fn version(&self) -> usize {
            self.version
        }

        fn created_at(&self) -> Option<DateTime<Utc>> {
            self.created_at
        }

        fn created_by(&self) -> Option<String> {
            None
        }

        fn last_modified_at(&self) -> Option<DateTime<Utc>> {
            self.last_modified_at
        }

        fn last_modified_by(&self) -> Option<String> {
            None
        }
    }

    #[derive(Debug)]
    enum UserSpecification {
        Id(HashSet<String>),
        Name(HashSet<String>),
        Manager(Option<HashSet<String>>),
    }

    impl Specification for UserSpecification {}

    #[derive(Debug, Default)]
    struct UserRepository(HashMap<String, UserPO>);

    #[async_trait::async_trait]
    impl InMemoryReadRepository for UserRepository {
        type Persistence = UserPO;

        fn satisfies(
            &self,
            persistence: &Self::Persistence,
            specification: &Self::Specification,
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
            let manager = if let Some(manager_id) = &persistence.manager_id {
                self.get_storage()
                    .get(manager_id)
                    .cloned()
                    .map(|po| self.convert_to_entity(po))
            } else {
                None
            };
            User {
                id: persistence.id,
                version: persistence.version,
                created_at: persistence.created_at,
                last_modified_at: persistence.last_modified_at,
                name: persistence.name,
                manager: manager.map(Arc::new),
            }
        }

        fn convert_to_persistence(&self, entity: &Self::Entity) -> Self::Persistence {
            UserPO {
                id: entity.id.clone(),
                version: entity.version,
                created_at: entity.created_at,
                last_modified_at: entity.last_modified_at,
                name: entity.name.clone(),
                manager_id: entity.manager.as_ref().map(|m| m.id.clone()),
            }
        }
    }

    #[async_trait::async_trait]
    impl InMemoryWriteRepository for UserRepository {}

    #[async_trait::async_trait]
    impl WriteRepository for UserRepository {
        async fn save_all(&mut self, entities: &[&Self::Entity]) -> HashSet<Self::Entity> {
            self.__save_all(entities).await
        }

        async fn delete_all_by_ids(&mut self, ids: &[&str]) -> HashSet<Self::Entity> {
            self.__delete_all_by_ids(ids).await
        }
    }

    #[async_trait::async_trait]
    impl ReadRepository for UserRepository {
        type Entity = User;
        type Specification = UserSpecification;

        async fn find_all_by_ids(&self, ids: &[&str]) -> HashSet<Self::Entity> {
            self.__find_all_by_ids(ids).await
        }

        async fn find_all(
            &self,
            spec: &Self::Specification,
            limit: Option<usize>,
        ) -> HashSet<Self::Entity> {
            self.__find_all(spec, limit).await
        }
    }

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let user_alice = Arc::new(User {
            id: "1".to_string(),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 1, 1, 0, 0, 0).unwrap()),
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 2, 1, 0, 0, 0).unwrap()),
            name: "Alice".to_string(),
            manager: None,
        });

        let user_bob = Arc::new(User {
            id: "2".to_string(),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 3, 1, 0, 0, 0).unwrap()),
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 4, 1, 0, 0, 0).unwrap()),
            name: "Bob".to_string(),
            manager: Some(user_alice.clone()),
        });

        let user_charlie = User {
            id: "3".to_string(),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 5, 1, 0, 0, 0).unwrap()),
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 6, 1, 0, 0, 0).unwrap()),
            name: "Charlie".to_string(),
            manager: Some(user_bob.clone()),
        };

        let user_betty = User {
            id: "4".to_string(),
            version: 1,
            created_at: Some(Utc.with_ymd_and_hms(2016, 7, 1, 0, 0, 0).unwrap()),
            last_modified_at: Some(Utc.with_ymd_and_hms(2016, 8, 1, 0, 0, 0).unwrap()),
            name: "Betty".to_string(),
            manager: Some(user_alice.clone()),
        };

        let user_repo = Arc::new(Mutex::new(UserRepository::default()));

        let mut user_repo = user_repo.lock().await;
        user_repo
            .save_all(&[
                user_alice.as_ref(),
                user_bob.as_ref(),
                &user_charlie,
                &user_betty,
            ])
            .await;

        let result = user_repo
            .find_all_by_ids(&[user_charlie.id.as_str(), user_betty.id.as_str()])
            .await;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&user_charlie));
        assert!(result.contains(&user_betty));

        let new_charlie = result.iter().find(|u| u.name == user_charlie.name).unwrap();
        assert_eq!(new_charlie.manager, user_charlie.manager);

        let result = user_repo
            .find_all(
                &UserSpecification::Manager(Some(HashSet::from_iter([user_alice.id.clone()]))),
                None,
            )
            .await;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&user_betty));
        assert!(result.contains(&user_bob));

        let result = user_repo
            .find_all(
                &UserSpecification::Name(HashSet::from_iter([
                    user_charlie.name.clone(),
                    user_bob.name.to_lowercase().clone(),
                    "Nonexistent".to_string(),
                ])),
                None,
            )
            .await;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&user_charlie));
        assert!(result.contains(&user_bob));

        let results = user_repo.delete_all_by_ids(&[user_bob.id()]).await;
        assert_eq!(results.len(), 1);
        assert!(results.contains(&user_bob));

        let result = user_repo.find_all_by_ids(&[user_charlie.id()]).await;
        assert_eq!(result.len(), 1);
        assert!(result.iter().next().unwrap().manager.is_none());

        Ok(())
    }
}
