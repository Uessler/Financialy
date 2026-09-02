use crate::{
    AppState,
    auth::CurrentUser,
    error::AppError,
    models::{Category, CategoryInput},
};
use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

fn validate(input: &CategoryInput) -> Result<(), AppError> {
    if input.name.trim().is_empty() || input.name.chars().count() > 60 {
        return Err(AppError::Validation("nome da categoria inválido".into()));
    }
    if !matches!(input.kind.as_str(), "income" | "expense") {
        return Err(AppError::Validation("tipo inválido".into()));
    }
    if input.color.len() != 7 || !input.color.starts_with('#') {
        return Err(AppError::Validation("cor inválida".into()));
    }
    Ok(())
}
pub async fn list(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
) -> Result<Json<Vec<Category>>, AppError> {
    Ok(Json(sqlx::query_as("SELECT id,name,color,kind::text AS kind,created_at FROM categories WHERE user_id=$1 ORDER BY name").bind(uid).fetch_all(&s.db).await?))
}
pub async fn create(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Json(i): Json<CategoryInput>,
) -> Result<Json<Category>, AppError> {
    validate(&i)?;
    Ok(Json(sqlx::query_as("INSERT INTO categories(user_id,name,color,kind) VALUES($1,$2,$3,$4::transaction_kind) RETURNING id,name,color,kind::text AS kind,created_at")
        .bind(uid).bind(i.name.trim()).bind(i.color).bind(i.kind).fetch_one(&s.db).await?))
}
pub async fn update(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Path(id): Path<Uuid>,
    Json(i): Json<CategoryInput>,
) -> Result<Json<Category>, AppError> {
    validate(&i)?;
    Ok(Json(sqlx::query_as("UPDATE categories SET name=$1,color=$2,kind=$3::transaction_kind WHERE id=$4 AND user_id=$5 RETURNING id,name,color,kind::text AS kind,created_at")
        .bind(i.name.trim()).bind(i.color).bind(i.kind).bind(id).bind(uid).fetch_one(&s.db).await?))
}
pub async fn delete(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let done = sqlx::query("DELETE FROM categories WHERE id=$1 AND user_id=$2")
        .bind(id)
        .bind(uid)
        .execute(&s.db)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
