#[cfg(test)]
mod test;

pub mod entity;
pub mod item_entity;
pub mod tag_entity;

use std::collections::{HashMap, HashSet};

use database_seaorm::repository::{SeaOrmReadRepository, SeaOrmSession, SeaOrmWriteRepository};
use domain::account::{AccountId, AccountType};
use domain::journal::JournalId;
use domain::record::repository::RecordRepository;
use domain::record::specification::{RecordSpec, RecordSpecification};
use domain::record::{
    Amount, Cost, Record, RecordId, RecordItemKind, RecordItemTransaction, RecordItemValidation,
    RecordItems,
};
use entity::RecordKindEnum;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Set};
use shared::{
    EntityId, Id, NonEmpty, NonNegative, ReadRepository, Result, SpecificationExpression,
    WriteRepository,
};

// ── Stateless Repository ─────────────────────────────────────────

#[derive(Default)]
pub struct SeaOrmRecordRepository;

impl SeaOrmRecordRepository {
    fn spec_leaf_to_condition(&self, spec: &RecordSpecification) -> Condition {
        match spec {
            RecordSpecification::Id(ids) => {
                let v: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                Condition::any().add(entity::Column::Id.is_in(v))
            }
            RecordSpecification::JournalId(ids) => {
                let v: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                Condition::any().add(entity::Column::JournalId.is_in(v))
            }
            RecordSpecification::AccountId(ids) => {
                let v: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();
                let subquery = item_entity::Entity::find()
                    .filter(item_entity::Column::AccountId.is_in(v))
                    .select_only()
                    .column(item_entity::Column::RecordId)
                    .into_query();
                Condition::any().add(entity::Column::Id.in_subquery(subquery))
            }
            RecordSpecification::Date(dates) => {
                let v: Vec<chrono::NaiveDate> = dates.iter().copied().collect();
                Condition::any().add(entity::Column::Date.is_in(v))
            }
            RecordSpecification::DateFrom(date) => {
                Condition::all().add(entity::Column::Date.gte(*date))
            }
            RecordSpecification::DateTo(date) => {
                Condition::all().add(entity::Column::Date.lte(*date))
            }
            RecordSpecification::Payee(payees) => {
                let v: Vec<String> = payees.iter().cloned().collect();
                Condition::any().add(entity::Column::Payee.is_in(v))
            }
            RecordSpecification::Tag(tags) => {
                let v: Vec<String> = tags.iter().cloned().collect();
                let subquery = tag_entity::Entity::find()
                    .filter(tag_entity::Column::Tag.is_in(v))
                    .select_only()
                    .column(tag_entity::Column::RecordId)
                    .into_query();
                Condition::any().add(entity::Column::Id.in_subquery(subquery))
            }
            RecordSpecification::ItemKind(kind) => {
                let kind_enum = Self::to_kind_enum(*kind);
                Condition::all().add(entity::Column::Kind.eq(kind_enum))
            }
            RecordSpecification::FullText(query) => {
                let pattern = format!("%{query}%");
                Condition::any()
                    .add(entity::Column::Description.contains(&pattern))
                    .add(entity::Column::Payee.contains(&pattern))
            }
        }
    }

    fn to_kind_enum(kind: RecordItemKind) -> RecordKindEnum {
        match kind {
            RecordItemKind::Transaction => RecordKindEnum::Transaction,
            RecordItemKind::Validation => RecordKindEnum::Validation,
        }
    }

    fn account_type_from_str(s: &str) -> AccountType {
        match s {
            "Asset" => AccountType::Asset,
            "Liability" => AccountType::Liability,
            "Equity" => AccountType::Equity,
            "Income" => AccountType::Income,
            "Expense" => AccountType::Expense,
            _ => AccountType::Asset,
        }
    }

    fn account_type_to_str(t: AccountType) -> String {
        match t {
            AccountType::Asset => "Asset".to_string(),
            AccountType::Liability => "Liability".to_string(),
            AccountType::Equity => "Equity".to_string(),
            AccountType::Income => "Income".to_string(),
            AccountType::Expense => "Expense".to_string(),
        }
    }

    fn parse_amount(number: &str, unit: &str) -> Amount {
        let decimal = number.parse::<rust_decimal::Decimal>().unwrap_or_default();
        Amount {
            amount: NonNegative::try_from(decimal).expect("stored amount should be non-negative"),
            unit: NonEmpty::try_from(unit.to_string()).expect("stored unit should be non-empty"),
        }
    }

    fn parse_costs(costs_json: &Option<String>) -> HashSet<Cost> {
        let Some(json_str) = costs_json else {
            return HashSet::new();
        };
        serde_json::from_str(json_str).unwrap_or_default()
    }

    fn serialize_costs(costs: &HashSet<Cost>) -> Option<String> {
        if costs.is_empty() {
            return None;
        }
        serde_json::to_string(costs).ok()
    }

    // ── Batch operations for related data ────────────────────────

    async fn load_tags_batch(
        &self,
        sess: &SeaOrmSession,
        record_ids: &[String],
    ) -> std::result::Result<HashMap<String, HashSet<String>>, sea_orm::DbErr> {
        if record_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids = record_ids.to_vec();
        let tags = sess
            .query_all(tag_entity::Entity::find().filter(tag_entity::Column::RecordId.is_in(ids)))
            .await?;

        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for tag in tags {
            result.entry(tag.record_id).or_default().insert(tag.tag);
        }
        Ok(result)
    }

    async fn load_items_batch(
        &self,
        sess: &SeaOrmSession,
        record_ids: &[String],
    ) -> std::result::Result<HashMap<String, Vec<item_entity::Model>>, sea_orm::DbErr> {
        if record_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids = record_ids.to_vec();
        let items = sess
            .query_all(item_entity::Entity::find().filter(item_entity::Column::RecordId.is_in(ids)))
            .await?;

        let mut result: HashMap<String, Vec<item_entity::Model>> = HashMap::new();
        for item in items {
            result.entry(item.record_id.clone()).or_default().push(item);
        }
        Ok(result)
    }

    async fn save_tags_batch(
        &self,
        sess: &SeaOrmSession,
        records: &[Record],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        if records.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = records.iter().map(|r| r.id.value().to_string()).collect();
        sess.exec_delete(
            tag_entity::Entity::delete_many().filter(tag_entity::Column::RecordId.is_in(ids)),
        )
        .await?;

        for record in records {
            let id = record.id.value().to_string();
            for tag in &record.tags {
                let tag_model = tag_entity::ActiveModel {
                    record_id: Set(id.clone()),
                    tag: Set(tag.to_string()),
                };
                sess.exec_insert(tag_entity::Entity::insert(tag_model))
                    .await?;
            }
        }
        Ok(())
    }

    async fn save_items_batch(
        &self,
        sess: &SeaOrmSession,
        records: &[Record],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        if records.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = records.iter().map(|r| r.id.value().to_string()).collect();
        sess.exec_delete(
            item_entity::Entity::delete_many().filter(item_entity::Column::RecordId.is_in(ids)),
        )
        .await?;

        for record in records {
            let record_id = record.id.value().to_string();
            match &record.items {
                RecordItems::Transactions(txns) => {
                    for txn in txns.iter() {
                        let item_model = item_entity::ActiveModel {
                            id: Set(uuid::Uuid::now_v7().to_string()),
                            record_id: Set(record_id.clone()),
                            account_id: Set(txn.account_id.value().to_string()),
                            account_type: Set(Some(Self::account_type_to_str(txn.account_type))),
                            amount_number: Set(txn.amount.amount.to_string()),
                            amount_unit: Set(txn.amount.unit.to_string()),
                            description: Set(txn.description.clone()),
                            price_number: Set(txn.price.as_ref().map(|p| p.amount.to_string())),
                            price_unit: Set(txn.price.as_ref().map(|p| p.unit.to_string())),
                            costs: Set(Self::serialize_costs(&txn.cost)),
                        };
                        sess.exec_insert(item_entity::Entity::insert(item_model))
                            .await?;
                    }
                }
                RecordItems::Validations(vals) => {
                    for val in vals.iter() {
                        let item_model = item_entity::ActiveModel {
                            id: Set(uuid::Uuid::now_v7().to_string()),
                            record_id: Set(record_id.clone()),
                            account_id: Set(val.account_id.value().to_string()),
                            account_type: Set(None),
                            amount_number: Set(val.amount.amount.to_string()),
                            amount_unit: Set(val.amount.unit.to_string()),
                            description: Set(val.description.clone()),
                            price_number: Set(None),
                            price_unit: Set(None),
                            costs: Set(None),
                        };
                        sess.exec_insert(item_entity::Entity::insert(item_model))
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn delete_related_batch(
        &self,
        sess: &SeaOrmSession,
        ids: &[Id<Record>],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        if ids.is_empty() {
            return Ok(());
        }
        let values: Vec<String> = ids.iter().map(|id| id.value().to_string()).collect();

        sess.exec_delete(
            item_entity::Entity::delete_many()
                .filter(item_entity::Column::RecordId.is_in(values.clone())),
        )
        .await?;

        sess.exec_delete(
            tag_entity::Entity::delete_many().filter(tag_entity::Column::RecordId.is_in(values)),
        )
        .await?;

        Ok(())
    }

    fn reconstruct_items(
        kind: &RecordKindEnum,
        item_models: Vec<item_entity::Model>,
    ) -> Result<RecordItems> {
        match kind {
            RecordKindEnum::Transaction => {
                let txns: Vec<RecordItemTransaction> = item_models
                    .into_iter()
                    .map(|m| RecordItemTransaction {
                        account_id: AccountId::from(m.account_id),
                        account_type: m
                            .account_type
                            .as_deref()
                            .map(Self::account_type_from_str)
                            .unwrap_or_default(),
                        amount: Self::parse_amount(&m.amount_number, &m.amount_unit),
                        description: m.description,
                        price: match (&m.price_number, &m.price_unit) {
                            (Some(n), Some(u)) => Some(Self::parse_amount(n, u)),
                            _ => None,
                        },
                        cost: Self::parse_costs(&m.costs),
                    })
                    .collect();
                Ok(RecordItems::Transactions(
                    NonEmpty::try_from(txns).map_err(|e| e.convert())?,
                ))
            }
            RecordKindEnum::Validation => {
                let vals: Vec<RecordItemValidation> = item_models
                    .into_iter()
                    .map(|m| RecordItemValidation {
                        account_id: AccountId::from(m.account_id),
                        amount: Self::parse_amount(&m.amount_number, &m.amount_unit),
                        description: m.description,
                    })
                    .collect();
                Ok(RecordItems::Validations(
                    NonEmpty::try_from(vals).map_err(|e| e.convert())?,
                ))
            }
        }
    }
}

// ── SeaOrmReadRepository ─────────────────────────────────────────

#[async_trait::async_trait]
impl SeaOrmReadRepository<RecordSpec> for SeaOrmRecordRepository {
    type SeaOrmEntity = entity::Entity;
    type SeaOrmActiveModel = entity::ActiveModel;

    async fn convert_to_entities(
        &self,
        sess: &SeaOrmSession,
        models: Vec<entity::Model>,
    ) -> Result<Vec<Record>> {
        let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        let tags_by_id = self
            .load_tags_batch(sess, &ids)
            .await
            .map_err(shared::ErrorKind::internal)?;
        let items_by_id = self
            .load_items_batch(sess, &ids)
            .await
            .map_err(shared::ErrorKind::internal)?;

        models
            .into_iter()
            .map(|model| {
                let tags = tags_by_id.get(&model.id).cloned().unwrap_or_default();
                let item_models = items_by_id.get(&model.id).cloned().unwrap_or_default();
                let items = Self::reconstruct_items(&model.kind, item_models)?;

                Ok(Record {
                    id: RecordId::from(model.id),
                    version: model.version as usize,
                    created_at: model.created_at,
                    last_modified_at: model.last_modified_at,
                    journal_id: JournalId::from(model.journal_id),
                    date: model.date,
                    items,
                    description: model.description,
                    tags: tags.into_iter().filter_map(|t| t.try_into().ok()).collect(),
                    payee: model.payee,
                })
            })
            .collect()
    }

    fn convert_to_active_model(&self, record: &Record) -> entity::ActiveModel {
        let kind = match &record.items {
            RecordItems::Transactions(_) => RecordKindEnum::Transaction,
            RecordItems::Validations(_) => RecordKindEnum::Validation,
        };
        entity::ActiveModel {
            id: Set(record.id.value().to_string()),
            version: Set(record.version as i32),
            created_at: Set(record.created_at),
            last_modified_at: Set(record.last_modified_at),
            journal_id: Set(record.journal_id.value().to_string()),
            date: Set(record.date),
            kind: Set(kind),
            description: Set(record.description.clone()),
            payee: Set(record.payee.clone()),
        }
    }

    fn spec_to_condition(&self, spec: &RecordSpec) -> Condition {
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

    fn id_to_value(&self, id: &Id<Record>) -> sea_orm::Value {
        sea_orm::Value::from(id.value().to_string())
    }

    fn pk_column(&self) -> entity::Column {
        entity::Column::Id
    }
}

// ── SeaOrmWriteRepository ────────────────────────────────────────

#[async_trait::async_trait]
impl SeaOrmWriteRepository<RecordSpec> for SeaOrmRecordRepository {
    async fn save_all_related(
        &self,
        sess: &SeaOrmSession,
        entities: &[Record],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        self.save_tags_batch(sess, entities).await?;
        self.save_items_batch(sess, entities).await?;
        Ok(())
    }

    async fn delete_all_related(
        &self,
        sess: &SeaOrmSession,
        ids: &[Id<Record>],
    ) -> std::result::Result<(), sea_orm::DbErr> {
        self.delete_related_batch(sess, ids).await
    }

    fn updatable_columns(&self) -> Vec<entity::Column> {
        vec![
            entity::Column::Version,
            entity::Column::CreatedAt,
            entity::Column::LastModifiedAt,
            entity::Column::Date,
            entity::Column::Description,
            entity::Column::Payee,
        ]
    }
}

// ── ReadRepository / WriteRepository impls ───────────────────────

#[async_trait::async_trait]
impl ReadRepository<RecordSpec> for SeaOrmRecordRepository {
    type Entity = Record;
    type Session = SeaOrmSession;

    async fn find_all_by_ids(
        &self,
        sess: &Self::Session,
        ids: &[Id<Record>],
    ) -> Result<HashMap<Id<Record>, Record>> {
        self.__find_all_by_ids(sess, ids).await
    }

    async fn find_all(
        &self,
        sess: &Self::Session,
        spec: &RecordSpec,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<Record>, Record>> {
        self.__find_all(sess, spec, limit).await
    }
}

#[async_trait::async_trait]
impl WriteRepository<RecordSpec> for SeaOrmRecordRepository {
    async fn save_all(
        &self,
        sess: &mut Self::Session,
        entities: &[Record],
    ) -> Result<HashMap<Id<Record>, Record>> {
        self.__save_all(sess, entities).await
    }

    async fn delete_all_by_ids(
        &self,
        sess: &mut Self::Session,
        ids: &[Id<Record>],
    ) -> Result<Vec<Id<Record>>> {
        self.__delete_all_by_ids(sess, ids).await
    }
}

impl RecordRepository for SeaOrmRecordRepository {}
