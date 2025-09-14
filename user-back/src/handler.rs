use crate::{
    jwt_auth,
    model::{LoginUserSchema, RegisterUserSchema, User},
    response::FilteredUser,
    token, AppState,
};

use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    get, post, web, HttpRequest, HttpResponse, Responder,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono;
use redis::AsyncCommands;
use sqlx::Row;
use uuid::Uuid;

use std::time::Instant;
use sysinfo::System;

#[get("/healthchecker")]
async fn health_checker_handler(data: web::Data<AppState>) -> impl Responder {
    let start_time = Instant::now();

    // Проверяем соединение с Redis
    let redis_healthy = match data.redis_client.get_async_connection().await {
        Ok(mut conn) => match conn.set::<&str, &str, String>("healthcheck", "test").await {
            Ok(_) => true,
            Err(_) => false,
        },
        Err(_) => false,
    };

    // Получаем системную информацию
    let mut system = System::new_all();
    system.refresh_all();

    // Преобразуем u128 в u64 для JSON сериализации
    let response_time_ms = start_time.elapsed().as_millis() as u64;
    let memory_used_mb = system.used_memory() / 1024 / 1024;
    let memory_total_mb = system.total_memory() / 1024 / 1024;

    let response = serde_json::json!({
        "status": if redis_healthy { "healthy" } else { "degraded" },
        "message": "Actix-web and Postgres: JWT RS256 Access and Refresh Tokens",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "response_time_ms": response_time_ms,
        "services": {
            "redis": redis_healthy
        },
        "system": {
            "memory_used_mb": memory_used_mb,
            "memory_total_mb": memory_total_mb,
            "cpu_usage_percent": system.global_cpu_usage() as f64,
        }
    });

    HttpResponse::Ok().json(response)
}

#[post("/auth/register")]
async fn register_user_handler(
    body: web::Json<RegisterUserSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let exists: bool = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(body.email.to_owned())
        .fetch_one(&data.db)
        .await
        .unwrap()
        .get(0);

    if exists {
        return HttpResponse::Conflict().json(
            serde_json::json!({"status": "fail","message": "User with that email already exists"}),
        );
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string();
    let query_result = sqlx::query_as!(
        User,
        "INSERT INTO users (name,email,password) VALUES ($1, $2, $3) RETURNING *",
        body.name.to_string(),
        body.email.to_string().to_lowercase(),
        hashed_password
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(user) => {
            let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "user": filter_user_record(&user)
            })});

            return HttpResponse::Ok().json(user_response);
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    }
}

#[post("/auth/login")]
async fn login_user_handler(
    body: web::Json<LoginUserSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let query_result = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", body.email)
        .fetch_optional(&data.db)
        .await
        .unwrap();

    let user = match query_result {
        Some(user) => user,
        None => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"status": "fail", "message": "Invalid email or password"}),
            );
        }
    };

    let is_valid = PasswordHash::new(&user.password)
        .and_then(|parsed_hash| {
            Argon2::default().verify_password(body.password.as_bytes(), &parsed_hash)
        })
        .map_or(false, |_| true);

    if !is_valid {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"status": "fail", "message": "Invalid email or password"}));
    }

    let access_token_details = match token::generate_jwt_token(
        user.id,
        data.env.access_token_max_age,
        data.env.access_token_private_key.clone(),
        "access",
        &data.env,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let refresh_token_details = match token::generate_jwt_token(
        user.id,
        data.env.refresh_token_max_age,
        data.env.refresh_token_private_key.clone(),
        "refresh",
        &data.env,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let mut redis_client = match data.redis_client.get_async_connection().await {
        Ok(redis_client) => redis_client,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let access_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.env.access_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = access_result {
        return HttpResponse::UnprocessableEntity()
            .json(serde_json::json!({"status": "error", "message": format!("{}", e)}));
    }

    let refresh_result: redis::RedisResult<()> = redis_client
        .set_ex(
            refresh_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.env.refresh_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = refresh_result {
        return HttpResponse::UnprocessableEntity()
            .json(serde_json::json!({"status": "error", "message": format!("{}", e)}));
    }

    let access_cookie = Cookie::build("access_token", access_token_details.token.clone().unwrap())
        .path("/")
        .max_age(ActixWebDuration::new(data.env.access_token_max_age * 60, 0))
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build(
        "refresh_token",
        refresh_token_details.token.clone().unwrap(),
    )
    .path("/")
    .max_age(ActixWebDuration::new(
        data.env.refresh_token_max_age * 60,
        0,
    ))
    .http_only(true)
    .finish();
    let logged_in_cookie = Cookie::build("logged_in", "true")
        .path("/")
        .max_age(ActixWebDuration::new(data.env.access_token_max_age * 60, 0))
        .http_only(false)
        .finish();

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(logged_in_cookie)
        .json(serde_json::json!({
            "status": "success",
            "access_token": access_token_details.token.unwrap(),
            "refresh_token": refresh_token_details.token.unwrap()
        }))
}

#[get("/auth/refresh")]
async fn refresh_access_token_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let message = "could not refresh access token";

    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": message}));
        }
    };

    let mut redis_client = match data.redis_client.get_async_connection().await {
        Ok(redis_client) => redis_client,
        Err(e) => {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"status": "fail", "message": format!("Could not connect to Redis: {}", e)}),
            );
        }
    };

    let refresh_token_details = match token::verify_jwt_token(
        data.env.refresh_token_public_key.clone(),
        &refresh_token,
        &data.env,
        &mut redis_client,
    )
    .await
    {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format!("{:?}", e)}));
        }
    };

    let redis_result: redis::RedisResult<String> = redis_client
        .get(refresh_token_details.token_uuid.to_string())
        .await;

    let user_id = match redis_result {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": message}));
        }
    };

    let user_id_uuid = Uuid::parse_str(&user_id).unwrap();
    let query_result = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id_uuid)
        .fetch_optional(&data.db)
        .await
        .unwrap();

    if query_result.is_none() {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"status": "fail", "message": "the user belonging to this token no longer exists"}));
    }

    let user = query_result.unwrap();

    let access_token_details = match token::generate_jwt_token(
        user.id,
        data.env.access_token_max_age,
        data.env.access_token_private_key.clone(),
        "access",
        &data.env,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("{:?}", e)}));
        }
    };

    let new_refresh_token_details = match token::generate_jwt_token(
        user.id,
        data.env.refresh_token_max_age,
        data.env.refresh_token_private_key.clone(),
        "refresh",
        &data.env,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("{:?}", e)}));
        }
    };

    let access_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.env.access_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = access_result {
        return HttpResponse::UnprocessableEntity()
            .json(serde_json::json!({"status": "error", "message": format!("{}", e)}));
    }

    let refresh_result: redis::RedisResult<()> = redis_client
        .set_ex(
            new_refresh_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.env.refresh_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = refresh_result {
        return HttpResponse::UnprocessableEntity()
            .json(serde_json::json!({"status": "error", "message": format!("{}", e)}));
    }

    let _: redis::RedisResult<()> = redis_client
        .del(refresh_token_details.token_uuid.to_string())
        .await;

    let access_cookie = Cookie::build("access_token", access_token_details.token.clone().unwrap())
        .path("/")
        .max_age(ActixWebDuration::new(data.env.access_token_max_age * 60, 0))
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build(
        "refresh_token",
        new_refresh_token_details.token.clone().unwrap(),
    )
    .path("/")
    .max_age(ActixWebDuration::new(
        data.env.refresh_token_max_age * 60,
        0,
    ))
    .http_only(true)
    .finish();
    let logged_in_cookie = Cookie::build("logged_in", "true")
        .path("/")
        .max_age(ActixWebDuration::new(data.env.access_token_max_age * 60, 0))
        .http_only(false)
        .finish();

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(logged_in_cookie)
        .json(serde_json::json!({
            "status": "success",
            "access_token": access_token_details.token.unwrap(),
            "refresh_token": new_refresh_token_details.token.unwrap()
        }))
}

#[get("/auth/logout")]
async fn logout_handler(
    req: HttpRequest,
    auth_guard: jwt_auth::JwtMiddleware,
    data: web::Data<AppState>,
) -> impl Responder {
    let message = "Token is invalid or session has expired";

    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": message}));
        }
    };

    let mut redis_client = match data.redis_client.get_async_connection().await {
        Ok(client) => client,
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"status": "error", "message": format!("Redis connection error: {}", e)}),
            );
        }
    };

    let refresh_token_details = match token::verify_jwt_token(
        data.env.refresh_token_public_key.clone(),
        &refresh_token,
        &data.env,
        &mut redis_client,
    )
    .await
    {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format!("{:?}", e)}));
        }
    };

    let redis_result: redis::RedisResult<usize> = redis_client
        .del(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.access_token_uuid.to_string(),
        ])
        .await;

    if redis_result.is_err() {
        return HttpResponse::UnprocessableEntity().json(
            serde_json::json!({"status": "error", "message": format!("{:?}", redis_result.unwrap_err())}),
        );
    }

    let access_cookie = Cookie::build("access_token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    let logged_in_cookie = Cookie::build("logged_in", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(logged_in_cookie)
        .json(serde_json::json!({"status": "success"}))
}

#[get("/users/me")]
async fn get_me_handler(jwt: jwt_auth::JwtMiddleware, data: web::Data<AppState>) -> impl Responder {

    let access_token_details = match token::generate_jwt_token(
        jwt.user.id,
        data.env.access_token_max_age,
        data.env.access_token_private_key.clone(),
        "access",
        &data.env,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let mut redis_client = match data.redis_client.get_async_connection().await {
        Ok(client) => client,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let _: redis::RedisResult<()> = redis_client.del(jwt.access_token_uuid.to_string()).await;

    let access_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            jwt.user.id.to_string(),
            (data.env.access_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = access_result {
        return HttpResponse::UnprocessableEntity()
            .json(serde_json::json!({"status": "error", "message": format!("{}", e)}));
    }

    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "user": filter_user_record(&jwt.user)
        }),
        "access_token": access_token_details.token.clone().unwrap()
    });

    HttpResponse::Ok()
        .cookie(
            Cookie::build("access_token", access_token_details.token.unwrap())
                .path("/")
                .max_age(ActixWebDuration::new(data.env.access_token_max_age * 60, 0))
                .http_only(true)
                .finish(),
        )
        .json(json_response)
}

#[get("/admins/verif")]
async fn get_admin_handler(
    jwt: jwt_auth::JwtMiddleware,
    data: web::Data<AppState>,
) -> impl Responder {
    if jwt.user.role != "admin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "status": "fail",
            "message": "Access denied: admin privileges required"
        }));
    }

    let access_token_details = match token::generate_jwt_token(
        jwt.user.id,
        data.env.access_token_max_age,
        data.env.access_token_private_key.clone(),
        "access",
        &data.env,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let mut redis_client = match data.redis_client.get_async_connection().await {
        Ok(client) => client,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "fail", "message": format!("{}", e)}));
        }
    };

    let _: redis::RedisResult<()> = redis_client.del(jwt.access_token_uuid.to_string()).await;

    let access_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            jwt.user.id.to_string(),
            (data.env.access_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = access_result {
        return HttpResponse::UnprocessableEntity()
            .json(serde_json::json!({"status": "error", "message": format!("{}", e)}));
    }

    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "user": filter_user_record(&jwt.user),
            "admin_verification": true
        }),
        "access_token": access_token_details.token.clone().unwrap()
    });

    HttpResponse::Ok()
        .cookie(
            Cookie::build("access_token", access_token_details.token.unwrap())
                .path("/")
                .max_age(ActixWebDuration::new(data.env.access_token_max_age * 60, 0))
                .http_only(true)
                .finish(),
        )
        .json(json_response)
}

fn filter_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        role: user.role.to_owned(),
        verified: user.verified,
        createdAt: user.created_at.unwrap(),
        updatedAt: user.updated_at.unwrap(),
    }
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(health_checker_handler)
        .service(register_user_handler)
        .service(login_user_handler)
        .service(refresh_access_token_handler)
        .service(logout_handler)
        .service(get_me_handler)
        .service(get_admin_handler);

    conf.service(scope);
}
