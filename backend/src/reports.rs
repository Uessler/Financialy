use crate::{
    AppState,
    auth::CurrentUser,
    error::AppError,
    models::{CategoryPoint, Dashboard, MonthlyPoint},
};
use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Period {
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
}
pub async fn dashboard(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Query(p): Query<Period>,
) -> Result<Json<Dashboard>, AppError> {
    let today = Utc::now().date_naive();
    let start = p
        .start
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap());
    let end = p.end.unwrap_or(today);
    if start > end {
        return Err(AppError::Validation("período inválido".into()));
    }
    let totals: (i64, i64) = sqlx::query_as(
        r#"SELECT
      COALESCE(SUM(amount_cents) FILTER(WHERE kind='income'),0)::bigint,
      COALESCE(SUM(amount_cents) FILTER(WHERE kind='expense'),0)::bigint
      FROM transactions WHERE user_id=$1 AND transaction_date BETWEEN $2 AND $3"#,
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .fetch_one(&s.db)
    .await?;
    let monthly=sqlx::query_as::<_,MonthlyPoint>(r#"SELECT date_trunc('month',transaction_date)::date AS month,
      COALESCE(SUM(amount_cents) FILTER(WHERE kind='income'),0)::bigint AS income_cents,
      COALESCE(SUM(amount_cents) FILTER(WHERE kind='expense'),0)::bigint AS expense_cents
      FROM transactions WHERE user_id=$1 AND transaction_date >= ($2::date - interval '5 months') AND transaction_date <= $3
      GROUP BY 1 ORDER BY 1"#).bind(uid).bind(start).bind(end).fetch_all(&s.db).await?;
    let by_category=sqlx::query_as::<_,CategoryPoint>(r#"SELECT COALESCE(c.name,'Sem categoria') AS category,
      COALESCE(c.color,'#94a3b8') AS color,SUM(t.amount_cents)::bigint AS total_cents
      FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
      WHERE t.user_id=$1 AND t.kind='expense' AND t.transaction_date BETWEEN $2 AND $3 GROUP BY c.name,c.color ORDER BY total_cents DESC"#)
      .bind(uid).bind(start).bind(end).fetch_all(&s.db).await?;
    Ok(Json(Dashboard {
        income_cents: totals.0,
        expense_cents: totals.1,
        balance_cents: totals.0 - totals.1,
        monthly,
        by_category,
    }))
}
