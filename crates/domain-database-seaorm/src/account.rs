#[cfg(test)]
mod test;

pub mod entity;
pub mod tag_entity;

use std::collections::{HashMap, HashSet};

use database_seaorm::repository::{SeaOrmReadRepository, SeaOrmSession, SeaOrmWriteRepository};
use domain::account::repository::AccountRepository;
use domain::account::specification::{AccountSpec, AccountSpecification};
use domain::account::{Account, AccountId, AccountType};
use domain::journal::JournalId;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Func;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, ExprTrait, QueryFilter, QuerySelect, QueryTrait, Set,
};
use shared::{EntityId, Id, ReadRepository, Result, SpecificationExpression, WriteRepository};

use entity::AccountTypeEnum;

// ── Stateless Repository ─────────────────────────────────────────

#[derive(Default)]
pub struct SeaOrmAccountRepository;

impl SeaOrmAccountRepository {
    fn spec_leaf_to_condition(&self, spec: &AccountSpecification) -> Condition {
        match spec {
            AccountSpecification::Id(ids) => {
                let v: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                Condition::any().add(entity::Column::Id.is_in(v))
            }
            AccountSpecification::JournalId(ids) => {
                let v: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                Condition::any().add(entity::Column::JournalId.is_in(v))
            }
            AccountSpecification::ParentId(ids) => {
                let v: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                Condition::any().add(entity::Column::ParentId.is_in(v))
            }
            AccountSpecification::Name(names) => {
                let v: Vec<String> = names.iter().map(|s| s.to_lowercase()).collect();
                Condition::any().add(Func::lower(Expr::col(entity::Column::Name)).is_in(v))
            }
            AccountSpecification::Type(types) => {
                let v: Vec<AccountTypeEnum> = types.iter().map(|t| Self::to_enum(*t)).collect();
                Condition::any().add(entity::Column::Type.is_in(v))
            }
            AccountSpecification::Tag(tags) => {
                let v: Vec<String> = tags.iter().map(|s| s.to_lowercase()).collect();
                let subquery = tag_entity::Entity::find()
                    .filter(Func::lower(Expr::col(tag_entity::Column::Tag)).is_in(v))
                    .select_only()
                    .column(tag_entity::Column::AccountId)
                    .into_query();
                Condition::any().add(entity::Column::Id.in_subquery(subquery))
            }
            AccountSpecification::FullText(query) => {
                let pattern = format!("%{}%", query.to_lowercase());
                Condition::any()
                    .add(Func::lower(Expr::col(entity::Column::Name)).like(&pattern))
                    .add(Func::lower(Expr::col(entity::Column::Description)).like(&pattern))
            }
            AccountSpecification::Archived(archived) => {
                if *archived {
                    Condition::all().add(entity::Column::ArchivedAt.is_not_null())
                } else {
                    Condition::all().add(entity::Column::ArchivedAt.is_null())
                }
            }
        }
    }

    fn from_enum(e: &AccountTypeEnum) -> AccountType {
        match e {
            AccountTypeEnum::Asset => AccountType::Asset,
            AccountTypeEnum::Liability => AccountType::Liability,
            AccountTypeEnum::Equity => AccountType::Equity,
            AccountTypeEnum::Income => AccountType::Income,
            AccountTypeEnum::Expense => AccountType::Expense,
        }
    }

    fn to_enum(t: AccountType) -> AccountTypeEnum {
        match t {
            AccountType::Asset => AccountTypeEnum::Asset,
            AccountType::Liability => AccountTypeEnum::Liability,
            AccountType::Equity => AccountTypeEnum::Equity,
            AccountType::Income => AccountTypeEnum::Income,
            AccountType::Expense => AccountTypeEnum::Expense,
        }
    }

    async fn load_tags_batch(
        &self,
        sess: &SeaOrmSession,
        account_ids: &[String],
    ) -> std::result::Result<HashMap<String, HashSet<String>>, sea_orm::DbErr> {
        if account_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let ids = account_ids.to_vec();
        let tags = sess
            .query_all(tag_entity::Entity::find().filter(tag_entity::Column::AccountId.is_in(ids)))
            .await?;

        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for tag in tags {
            result.entry(tag.account_id).or_default().insert(tag.tag);
        }
        Ok(result)
    }

    async fn save_tags_batch(
        &self,
        sess: &SeaOrmSession,
        accounts: &[Account],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        if accounts.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = accounts.iter().map(|a| a.id.value().to_string()).collect();

        sess.exec_delete(
            tag_entity::Entity::delete_many().filter(tag_entity::Column::AccountId.is_in(ids)),
        )
        .await?;

        for account in accounts {
            let id = account.id.value().to_string();
            for tag in &account.tags {
                let tag_model = tag_entity::ActiveModel {
                    account_id: Set(id.clone()),
                    tag: Set(tag.to_string()),
                };
                sess.exec_insert(tag_entity::Entity::insert(tag_model))
                    .await?;
            }
        }
        Ok(())
    }

    async fn delete_tags_batch(
        &self,
        sess: &SeaOrmSession,
        ids: &[Id<Account>],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        if ids.is_empty() {
            return Ok(());
        }

        let values: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
        sess.exec_delete(
            tag_entity::Entity::delete_many().filter(tag_entity::Column::AccountId.is_in(values)),
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl SeaOrmReadRepository<AccountSpec> for SeaOrmAccountRepository {
    type SeaOrmEntity = entity::Entity;
    type SeaOrmActiveModel = entity::ActiveModel;

    async fn convert_to_entities(
        &self,
        sess: &SeaOrmSession,
        models: Vec<entity::Model>,
    ) -> Result<Vec<Account>> {
        let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        let tags_by_id = self
            .load_tags_batch(sess, &ids)
            .await
            .map_err(shared::ErrorKind::internal)?;

        models
            .into_iter()
            .map(|model| {
                let tags = tags_by_id.get(&model.id).cloned().unwrap_or_default();
                Ok(Account {
                    id: AccountId::from(model.id),
                    version: model.version as usize,
                    created_at: model.created_at,
                    last_modified_at: model.last_modified_at,
                    archived_at: model.archived_at,
                    journal_id: JournalId::from(model.journal_id),
                    parent_id: model.parent_id.map(AccountId::from),
                    r#type: Self::from_enum(&model.r#type),
                    name: model
                        .name
                        .try_into()
                        .map_err(|e: shared::Error| e.convert())?,
                    description: model.description,
                    tags: tags.into_iter().filter_map(|t| t.try_into().ok()).collect(),
                })
            })
            .collect()
    }

    fn convert_to_active_model(&self, account: &Account) -> entity::ActiveModel {
        entity::ActiveModel {
            id: Set(account.id.value().to_string()),
            version: Set(account.version as i32),
            created_at: Set(account.created_at),
            last_modified_at: Set(account.last_modified_at),
            archived_at: Set(account.archived_at),
            journal_id: Set(account.journal_id.value().to_string()),
            parent_id: Set(account.parent_id.as_ref().map(|id| id.value().to_string())),
            r#type: Set(Self::to_enum(account.r#type)),
            name: Set(account.name.to_string()),
            description: Set(account.description.clone()),
        }
    }

    fn spec_to_condition(&self, spec: &AccountSpec) -> Condition {
        match spec {
            SpecificationExpression::Leaf(leaf) => self.spec_leaf_to_condition(leaf),
            SpecificationExpression::All(specs) => {
                let mut cond = Condition::all();
                for s in specs {
                    cond = cond.add(self.spec_to_condition(s));
                }
                cond
            }
            SpecificationExpression::Any(specs) => {
                let mut cond = Condition::any();
                for s in specs {
                    cond = cond.add(self.spec_to_condition(s));
                }
                cond
            }
            SpecificationExpression::Not(inner) => {
                Condition::all().add(self.spec_to_condition(inner).not())
            }
        }
    }

    fn id_to_value(&self, id: &Id<Account>) -> sea_orm::Value {
        sea_orm::Value::from(id.value().to_string())
    }

    fn pk_column(&self) -> entity::Column {
        entity::Column::Id
    }
}

#[async_trait::async_trait]
impl SeaOrmWriteRepository<AccountSpec> for SeaOrmAccountRepository {
    async fn save_all_related(
        &self,
        sess: &SeaOrmSession,
        entities: &[Account],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        self.save_tags_batch(sess, entities).await
    }

    async fn delete_all_related(
        &self,
        sess: &SeaOrmSession,
        ids: &[Id<Account>],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        self.delete_tags_batch(sess, ids).await
    }

    fn convert_to_temp_active_model(&self, account: &Account) -> entity::ActiveModel {
        entity::ActiveModel {
            id: Set(account.id.value().to_string()),
            version: Set(account.version as i32),
            created_at: Set(account.created_at),
            last_modified_at: Set(account.last_modified_at),
            archived_at: Set(account.archived_at),
            journal_id: Set(account.journal_id.value().to_string()),
            parent_id: Set(account.parent_id.as_ref().map(|id| id.value().to_string())),
            r#type: Set(Self::to_enum(account.r#type)),
            name: Set(format!("__temp__{}", uuid::Uuid::now_v7())),
            description: Set(account.description.clone()),
        }
    }

    fn updatable_columns(&self) -> Vec<entity::Column> {
        vec![
            entity::Column::Version,
            entity::Column::CreatedAt,
            entity::Column::LastModifiedAt,
            entity::Column::ArchivedAt,
            entity::Column::Name,
            entity::Column::Description,
        ]
    }
}

// ── ReadRepository / WriteRepository impls ───────────────────────

#[async_trait::async_trait]
impl ReadRepository<AccountSpec> for SeaOrmAccountRepository {
    type Entity = Account;
    type Session = SeaOrmSession;

    async fn find_all_by_ids(
        &self,
        sess: &Self::Session,
        ids: &[Id<Account>],
    ) -> Result<HashMap<Id<Account>, Account>> {
        self.__find_all_by_ids(sess, ids).await
    }

    async fn find_all(
        &self,
        sess: &Self::Session,
        spec: &AccountSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Account>, Account>> {
        self.__find_all(sess, spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<AccountSpec> for SeaOrmAccountRepository {
    async fn save_all(
        &self,
        sess: &mut Self::Session,
        entities: &[Account],
    ) -> Result<HashMap<Id<Account>, Account>> {
        self.__save_all(sess, entities).await
    }

    async fn delete_all_by_ids(
        &self,
        sess: &mut Self::Session,
        ids: &[Id<Account>],
    ) -> Result<Vec<Id<Account>>> {
        self.__delete_all_by_ids(sess, ids).await
    }
}

impl AccountRepository for SeaOrmAccountRepository {}
