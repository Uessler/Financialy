use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}
#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub kind: String,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
pub struct CategoryInput {
    pub name: String,
    pub color: String,
    pub kind: String,
}
#[derive(Debug, Serialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub kind: String,
    pub description: String,
    pub amount_cents: i64,
    pub transaction_date: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
pub struct TransactionInput {
    pub category_id: Option<Uuid>,
    pub kind: String,
    pub description: String,
    pub amount_cents: i64,
    pub transaction_date: NaiveDate,
    pub notes: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct TransactionFilter {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    pub kind: Option<String>,
}
#[derive(Debug, Serialize, FromRow)]
pub struct MonthlyPoint {
    pub month: NaiveDate,
    pub income_cents: i64,
    pub expense_cents: i64,
}
#[derive(Debug, Serialize, FromRow)]
pub struct CategoryPoint {
    pub category: String,
    pub color: String,
    pub total_cents: i64,
}
#[derive(Debug, Serialize)]
pub struct Dashboard {
    pub income_cents: i64,
    pub expense_cents: i64,
    pub balance_cents: i64,
    pub monthly: Vec<MonthlyPoint>,
    pub by_category: Vec<CategoryPoint>,
}
