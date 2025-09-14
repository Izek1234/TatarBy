use actix_web::cookie::{time::Duration as ActixWebDuration, Cookie};
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpRequest};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::User;
use crate::token;
use crate::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtMiddleware {
    pub user: User,
    pub access_token_uuid: Uuid,
    pub new_access_token: Option<String>,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = futures::future::BoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = match req.app_data::<web::Data<AppState>>() {
            Some(data) => data.clone(),
            None => {
                return Box::pin(async { Err(ErrorInternalServerError("AppState not found")) });
            }
        };

        let access_token = req
            .cookie("access_token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| {
                        if s.starts_with("Bearer ") {
                            Some(s[7..].to_string())
                        } else {
                            None
                        }
                    })
            });

        if access_token.is_none() {
            return Box::pin(async {
                Err(ErrorUnauthorized(
                    "You are not logged in, please provide token",
                ))
            });
        }

        let access_token_str = access_token.unwrap();

        Box::pin(async move {
            let mut redis_client = data
                .redis_client
                .get_async_connection()
                .await
                .map_err(|e| {
                    ErrorInternalServerError(format!("Could not connect to Redis: {}", e))
                })?;

            let access_token_details = match token::verify_jwt_token(
                data.env.access_token_public_key.clone(),
                &access_token_str,
                &data.env,
                &mut redis_client,
            )
            .await
            {
                Ok(token_details) => token_details,
                Err(e) => {
                    return Err(ErrorUnauthorized(format!("Invalid token: {:?}", e)));
                }
            };

            let access_token_uuid = access_token_details.token_uuid;

            let user_id: String = redis_client
                .get(access_token_uuid.to_string())
                .await
                .map_err(|_| ErrorUnauthorized("Token is invalid or session has expired"))?;

            let user_id_uuid = Uuid::parse_str(&user_id)
                .map_err(|_| ErrorUnauthorized("Invalid user ID in token"))?;

            let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id_uuid)
                .fetch_optional(&data.db)
                .await
                .map_err(|e| {
                    ErrorInternalServerError(format!("Failed to check user existence: {}", e))
                })?;

            match user {
                Some(user) => {
                    let new_access_token_details = match token::generate_jwt_token(
                        user.id,
                        5, 
                        data.env.access_token_private_key.clone(),
                        "access",
                        &data.env,
                    ) {
                        Ok(token_details) => token_details,
                        Err(e) => {
                            return Err(ErrorInternalServerError(format!(
                                "Failed to generate new token: {}",
                                e
                            )));
                        }
                    };

                    let set_result: redis::RedisResult<()> = redis_client
                        .set_ex(
                            new_access_token_details.token_uuid.to_string(),
                            user.id.to_string(),
                            300,
                        )
                        .await;

                    if let Err(e) = set_result {
                        return Err(ErrorInternalServerError(format!(
                            "Failed to save new token to Redis: {}",
                            e
                        )));
                    }

                    let _ = redis_client
                        .del::<&str, i32>(&access_token_uuid.to_string())
                        .await;

                    Ok(JwtMiddleware {
                        user,
                        access_token_uuid: new_access_token_details.token_uuid,
                        new_access_token: new_access_token_details.token,
                    })
                }
                None => Err(ErrorUnauthorized(
                    "The user belonging to this token no longer exists",
                )),
            }
        })
    }
}
