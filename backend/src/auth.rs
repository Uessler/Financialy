use crate::{AppState, error::AppError, models::User};
use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const COOKIE: &str = "financialy_session";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLogin {
    credential: String,
}

#[derive(Debug, Deserialize)]
struct GoogleToken {
    sub: String,
    email: String,
    name: String,
    picture: Option<String>,
    aud: String,
    email_verified: String,
}

pub struct CurrentUser(pub Uuid);

impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get(COOKIE).ok_or(AppError::Unauthorized)?.value();
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?
        .claims;
        Ok(Self(claims.sub))
    }
}

pub async fn google_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<GoogleLogin>,
) -> Result<(CookieJar, Json<User>), AppError> {
    let google: GoogleToken = state
        .http
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", &input.credential)])
        .send()
        .await
        .map_err(anyhow::Error::from)?
        .error_for_status()
        .map_err(|_| AppError::Unauthorized)?
        .json()
        .await
        .map_err(anyhow::Error::from)?;
    if google.aud != state.google_client_id || google.email_verified != "true" {
        return Err(AppError::Unauthorized);
    }
    let user = sqlx::query_as::<_, User>(r#"
        INSERT INTO users (google_subject, email, name, avatar_url) VALUES ($1,$2,$3,$4)
        ON CONFLICT (google_subject) DO UPDATE SET email=EXCLUDED.email,name=EXCLUDED.name,avatar_url=EXCLUDED.avatar_url,updated_at=now()
        RETURNING id,email,name,avatar_url"#)
        .bind(google.sub).bind(google.email).bind(google.name).bind(google.picture)
        .fetch_one(&state.db).await?;
    sqlx::query(
        r#"INSERT INTO categories (user_id,name,color,kind)
        SELECT $1,v.name,v.color,v.kind::transaction_kind FROM (VALUES
          ('Salário','#22a06b','income'),('Freelance','#5b4de3','income'),
          ('Alimentação','#ef625d','expense'),('Moradia','#6c5ce7','expense'),
          ('Transporte','#f59e0b','expense'),('Lazer','#ec4899','expense'),
          ('Saúde','#14b8a6','expense'),('Educação','#3b82f6','expense')
        ) AS v(name,color,kind) ON CONFLICT (user_id,name,kind) DO NOTHING"#,
    )
    .bind(user.id)
    .execute(&state.db)
    .await?;
    let now = Utc::now();
    let claims = Claims {
        sub: user.id,
        iat: now.timestamp() as usize,
        exp: (now + Duration::days(7)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(anyhow::Error::from)?;
    let cookie = Cookie::build((COOKIE, token))
        .http_only(true)
        .secure(state.cookie_secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::days(7))
        .build();
    Ok((jar.add(cookie), Json(user)))
}

pub async fn logout(jar: CookieJar) -> CookieJar {
    jar.remove(Cookie::build(COOKIE).path("/").build())
}

pub async fn me(
    State(state): State<AppState>,
    CurrentUser(id): CurrentUser,
) -> Result<Json<User>, AppError> {
    Ok(Json(
        sqlx::query_as("SELECT id,email,name,avatar_url FROM users WHERE id=$1")
            .bind(id)
            .fetch_one(&state.db)
            .await?,
    ))
}
