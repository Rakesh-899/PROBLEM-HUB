use actix_web::{post, get, put, web, HttpResponse, HttpRequest, Responder};
use sqlx::PgPool;
use crate::models::{LoginRequest, verify_password, generate_jwt, SignupRequest, hash_password, ResetPasswordRequest, verify_jwt};

#[get("/protected")]
pub async fn protected_route() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "You have accessed a protected route!"
    }))
}

#[post("/signup")]
pub async fn signup(
    pool: web::Data<PgPool>,
    payload: web::Json<SignupRequest>,
) -> impl Responder {
    let hashed = hash_password(&payload.password);

    let result = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        payload.username,
        payload.email,
        hashed
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Signup successful",
            "user_id": row.id
        })),
        Err(e) => {
            eprintln!("Signup error: {:?}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Could not create user (maybe username/email exists)"
            }))
        }
    }
}


#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, password_hash, username
        FROM users
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => {
            if verify_password(&payload.password, &user.password_hash) {
                let token = generate_jwt(user.id);
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Login successful",
                    "user_id": user.id,
                    "username": user.username,
                    "token": token
                }))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid email or password"
                }))
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid email or password"
        })),
        Err(e) => {
            eprintln!("Login error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Server error"
            }))
        }
    }
}

#[put("/reset-password")]
pub async fn reset_password(
    pool: web::Data<PgPool>,
    payload: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    let hashed = hash_password(&payload.new_password);

    

    let result = sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $1,
            updated_at = NOW()
        WHERE email = $2
        RETURNING id
        "#,
        hashed,
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Password updated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "No user found with that email"
        })),
        Err(e) => {
            eprintln!("Reset password error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Server error"
            }))
        }
    }
}

#[get("/me")]
pub async fn get_profile(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> impl Responder {
    // Extract token from Authorization header
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    if auth_header.is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing token"
        }));
    }
    let auth_header = auth_header.unwrap();

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid token format"
        }));
    }

    let token = &auth_header[7..];
    let claims = match verify_jwt(token) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or expired token"
            }));
        }
    };

    // Parse user_id from claims
    let user_id: i32 = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid user id in token"
            }));
        }
    };

    // Fetch user from DB
    let result = sqlx::query!(
        r#"
        SELECT id, username, email, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => HttpResponse::Ok().json(serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "created_at": user.created_at,
            "updated_at": user.updated_at
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => {
            eprintln!("Fetch user error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Server error"
            }))
        }
    }
}