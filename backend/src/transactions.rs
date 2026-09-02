use crate::{
    AppState,
    auth::CurrentUser,
    error::AppError,
    models::{Transaction, TransactionFilter, TransactionInput},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

fn validate(i: &TransactionInput) -> Result<(), AppError> {
    if i.description.trim().is_empty() || i.description.chars().count() > 120 {
        return Err(AppError::Validation("descrição inválida".into()));
    }
    if i.amount_cents <= 0 {
        return Err(AppError::Validation(
            "o valor deve ser maior que zero".into(),
        ));
    }
    if !matches!(i.kind.as_str(), "income" | "expense") {
        return Err(AppError::Validation("tipo inválido".into()));
    }
    if i.notes.as_ref().is_some_and(|n| n.chars().count() > 500) {
        return Err(AppError::Validation("observação muito longa".into()));
    }
    Ok(())
}
const SELECT: &str = "SELECT t.id,t.category_id,c.name AS category_name,t.kind::text AS kind,t.description,t.amount_cents,t.transaction_date,t.notes,t.created_at FROM transactions t LEFT JOIN categories c ON c.id=t.category_id";
pub async fn list(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Query(f): Query<TransactionFilter>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let q = format!(
        "{SELECT} WHERE t.user_id=$1 AND ($2::date IS NULL OR t.transaction_date >= $2) AND ($3::date IS NULL OR t.transaction_date <= $3) AND ($4::text IS NULL OR t.kind::text=$4) ORDER BY t.transaction_date DESC,t.created_at DESC LIMIT 500"
    );
    Ok(Json(
        sqlx::query_as(&q)
            .bind(uid)
            .bind(f.start)
            .bind(f.end)
            .bind(f.kind)
            .fetch_all(&s.db)
            .await?,
    ))
}
async fn category_ok(s: &AppState, uid: Uuid, i: &TransactionInput) -> Result<(), AppError> {
    if let Some(cid) = i.category_id {
        let ok: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id=$1 AND user_id=$2 AND kind::text=$3)",
        )
        .bind(cid)
        .bind(uid)
        .bind(&i.kind)
        .fetch_one(&s.db)
        .await?;
        if !ok {
            return Err(AppError::Validation("categoria incompatível".into()));
        }
    }
    Ok(())
}
pub async fn create(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Json(i): Json<TransactionInput>,
) -> Result<Json<Transaction>, AppError> {
    validate(&i)?;
    category_ok(&s, uid, &i).await?;
    let row: (Uuid,)=sqlx::query_as("INSERT INTO transactions(user_id,category_id,kind,description,amount_cents,transaction_date,notes) VALUES($1,$2,$3::transaction_kind,$4,$5,$6,$7) RETURNING id")
        .bind(uid).bind(i.category_id).bind(i.kind).bind(i.description.trim()).bind(i.amount_cents).bind(i.transaction_date).bind(i.notes).fetch_one(&s.db).await?;
    let q = format!("{SELECT} WHERE t.id=$1 AND t.user_id=$2");
    Ok(Json(
        sqlx::query_as(&q)
            .bind(row.0)
            .bind(uid)
            .fetch_one(&s.db)
            .await?,
    ))
}
pub async fn update(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Path(id): Path<Uuid>,
    Json(i): Json<TransactionInput>,
) -> Result<Json<Transaction>, AppError> {
    validate(&i)?;
    category_ok(&s, uid, &i).await?;
    let done=sqlx::query("UPDATE transactions SET category_id=$1,kind=$2::transaction_kind,description=$3,amount_cents=$4,transaction_date=$5,notes=$6,updated_at=now() WHERE id=$7 AND user_id=$8")
        .bind(i.category_id).bind(i.kind).bind(i.description.trim()).bind(i.amount_cents).bind(i.transaction_date).bind(i.notes).bind(id).bind(uid).execute(&s.db).await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    let q = format!("{SELECT} WHERE t.id=$1 AND t.user_id=$2");
    Ok(Json(
        sqlx::query_as(&q)
            .bind(id)
            .bind(uid)
            .fetch_one(&s.db)
            .await?,
    ))
}
pub async fn delete(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let done = sqlx::query("DELETE FROM transactions WHERE id=$1 AND user_id=$2")
        .bind(id)
        .bind(uid)
        .execute(&s.db)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    fn input(amount_cents: i64, kind: &str) -> TransactionInput {
        TransactionInput {
            category_id: None,
            kind: kind.into(),
            description: "Teste".into(),
            amount_cents,
            transaction_date: NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
            notes: None,
        }
    }
    #[test]
    fn accepts_valid_transaction() {
        assert!(validate(&input(1, "income")).is_ok());
    }
    #[test]
    fn rejects_non_positive_amount() {
        assert!(validate(&input(0, "expense")).is_err());
    }
    #[test]
    fn rejects_unknown_kind() {
        assert!(validate(&input(100, "transfer")).is_err());
    }
}
