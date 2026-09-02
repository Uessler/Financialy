use crate::{
    AppState,
    auth::CurrentUser,
    error::AppError,
    models::{
        CategoryPoint, Dashboard, DetailedReport, MonthlyPoint, ReportCategoryPoint, Transaction,
    },
};
use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Period {
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct ReportFilter {
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    kind: Option<String>,
    category_id: Option<Uuid>,
}

fn report_period(filter: &ReportFilter) -> Result<(NaiveDate, NaiveDate), AppError> {
    let today = Utc::now().date_naive();
    let start = filter
        .start
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap());
    let end = filter.end.unwrap_or(today);
    if start > end {
        return Err(AppError::Validation("período inválido".into()));
    }
    if filter
        .kind
        .as_deref()
        .is_some_and(|kind| !matches!(kind, "income" | "expense"))
    {
        return Err(AppError::Validation("tipo inválido".into()));
    }
    Ok((start, end))
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

pub async fn detailed(
    State(s): State<AppState>,
    CurrentUser(uid): CurrentUser,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<DetailedReport>, AppError> {
    let (start, end) = report_period(&filter)?;
    let totals: (i64, i64, i64) = sqlx::query_as(
        r#"SELECT
          COALESCE(SUM(amount_cents) FILTER(WHERE kind='income'),0)::bigint,
          COALESCE(SUM(amount_cents) FILTER(WHERE kind='expense'),0)::bigint,
          COUNT(*)::bigint
        FROM transactions
        WHERE user_id=$1 AND transaction_date BETWEEN $2 AND $3
          AND ($4::text IS NULL OR kind::text=$4)
          AND ($5::uuid IS NULL OR category_id=$5)"#,
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .bind(&filter.kind)
    .bind(filter.category_id)
    .fetch_one(&s.db)
    .await?;

    let by_category = sqlx::query_as::<_, ReportCategoryPoint>(
        r#"SELECT COALESCE(c.name,'Sem categoria') AS category,
          COALESCE(c.color,'#94a3b8') AS color,
          t.kind::text AS kind,
          SUM(t.amount_cents)::bigint AS total_cents
        FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
        WHERE t.user_id=$1 AND t.transaction_date BETWEEN $2 AND $3
          AND ($4::text IS NULL OR t.kind::text=$4)
          AND ($5::uuid IS NULL OR t.category_id=$5)
        GROUP BY c.name,c.color,t.kind ORDER BY total_cents DESC"#,
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .bind(&filter.kind)
    .bind(filter.category_id)
    .fetch_all(&s.db)
    .await?;

    let transactions = sqlx::query_as::<_, Transaction>(
        r#"SELECT t.id,t.category_id,c.name AS category_name,t.kind::text AS kind,
          t.description,t.amount_cents,t.transaction_date,t.notes,t.created_at
        FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
        WHERE t.user_id=$1 AND t.transaction_date BETWEEN $2 AND $3
          AND ($4::text IS NULL OR t.kind::text=$4)
          AND ($5::uuid IS NULL OR t.category_id=$5)
        ORDER BY t.transaction_date DESC,t.created_at DESC"#,
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .bind(&filter.kind)
    .bind(filter.category_id)
    .fetch_all(&s.db)
    .await?;

    Ok(Json(DetailedReport {
        start,
        end,
        income_cents: totals.0,
        expense_cents: totals.1,
        balance_cents: totals.0 - totals.1,
        transaction_count: totals.2,
        average_cents: if totals.2 == 0 {
            0
        } else {
            (totals.0 + totals.1) / totals.2
        },
        by_category,
        transactions,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(start: &str, end: &str, kind: Option<&str>) -> ReportFilter {
        ReportFilter {
            start: Some(start.parse().unwrap()),
            end: Some(end.parse().unwrap()),
            kind: kind.map(str::to_owned),
            category_id: None,
        }
    }

    #[test]
    fn accepts_valid_report_filter() {
        assert!(report_period(&filter("2026-01-01", "2026-01-31", Some("expense"))).is_ok());
    }

    #[test]
    fn rejects_inverted_period() {
        assert!(report_period(&filter("2026-02-01", "2026-01-31", None)).is_err());
    }

    #[test]
    fn rejects_unknown_kind() {
        assert!(report_period(&filter("2026-01-01", "2026-01-31", Some("transfer"))).is_err());
    }
}
