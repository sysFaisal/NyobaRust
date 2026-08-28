use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::c_auth::refresh_token::{AccesClaims, RoleModel};
use crate::error::error::AppError;

pub struct AuthUser {
    pub id: Uuid,
    pub role: RoleModel,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<AccesClaims>().ok_or_else(|| {
            AppError::Unauthorized(
                None,
                Some("auth_user: AccesClaims tidak ada di request".to_string()),
            )
        })?;

        let id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::InternalServerError(
                None,
                Some("auth_user: claims.sub bukan UUID valid".to_string()),
            )
        })?;

        Ok(AuthUser {
            id,
            role: claims.role,
        })
    }
}
