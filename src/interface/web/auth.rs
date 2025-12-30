use ntex::http::Payload;
use ntex::web::{self, HttpRequest};
use sea_orm::EntityTrait;

use crate::application::errors::{RepositoryError, UsecaseError};
use crate::domain::accelerator::AcceleratorUser;
use crate::infrastructure::persistence::{accelerator_user, accelerator_user_session};

use super::errors::AppError;
use super::AppState;

#[derive(Debug, Clone)]
pub struct AuthedAcceleratorUser {
    pub user: AcceleratorUser,
    pub token: String,
}

fn bearer_token(req: &HttpRequest) -> Option<String> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;
    let header = header.trim();
    let token = header.strip_prefix("Bearer ")?;
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

async fn load_user_by_token(state: &AppState, token: &str) -> Result<AcceleratorUser, AppError> {
    let now = chrono::Utc::now();
    let session = accelerator_user_session::Entity::find_by_id(token.to_string())
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(RepositoryError::Persistence(e.to_string()))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    let expires_at: chrono::DateTime<chrono::Utc> = session.expires_at.into();
    if expires_at <= now {
        return Err(UsecaseError::Unauthorized.into());
    }

    let user = accelerator_user::Entity::find_by_id(session.user_id)
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(RepositoryError::Persistence(e.to_string()))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    Ok(user.into())
}

impl<Err> web::FromRequest<Err> for AuthedAcceleratorUser
where
    Err: web::error::ErrorRenderer,
    AppError: Into<Err::Container>,
{
    type Error = AppError;

    async fn from_request(req: &HttpRequest, payload: &mut Payload) -> Result<Self, Self::Error> {
        let token = bearer_token(req).ok_or(UsecaseError::Unauthorized)?;
        let state = <web::types::State<AppState> as web::FromRequest<Err>>::from_request(
            req,
            payload,
        )
        .await
        .map_err(|_| UsecaseError::Unauthorized)?;

        let user = load_user_by_token(&state, &token).await?;
        Ok(Self { user, token })
    }
}
