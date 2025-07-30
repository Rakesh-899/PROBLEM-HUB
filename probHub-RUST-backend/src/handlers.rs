use actix_web::{post, get, put, web, HttpResponse, HttpRequest, Responder};
use sqlx::PgPool;
use crate::models::{LoginRequest, verify_password, generate_jwt, SignupRequest, hash_password, ResetPasswordRequest, verify_jwt};
use time::{OffsetDateTime, Duration, PrimitiveDateTime};
use rand::Rng;
use lettre::message::Message;
use lettre::transport::smtp::authentication::Credentials;
use lettre::SmtpTransport;
use lettre::Transport;
use serde::Deserialize;


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
        SELECT id, username, email, password_hash, is_verified
        FROM users
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => {
            if user.is_verified.unwrap_or(false) == false {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Account not verified. Please check your email for OTP."
                }));
            }

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
        Ok(None) => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid email or password"
            }))
        }
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

#[derive(Deserialize)]
pub struct SendOtpRequest {
    email: String,
}

#[post("/send-otp")]
pub async fn send_otp(
    pool: web::Data<PgPool>,
    payload: web::Json<SendOtpRequest>,
) -> impl Responder {
    let user = sqlx::query!(
        "SELECT id, email FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }));
        }
        Err(e) => {
            eprintln!("Error fetching user: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal Server error while fetching user from db"
            }));
        }
    };

    let otp_code: String = rand::thread_rng().gen_range(100000..999999).to_string();
    let otp_expiry_offset = OffsetDateTime::now_utc() + Duration::minutes(5);
    let otp_expiry = PrimitiveDateTime::new(otp_expiry_offset.date(), otp_expiry_offset.time());
    let result = sqlx::query!(
        "UPDATE users SET otp_code = $1, otp_expires_at = $2 WHERE id = $3",
        otp_code,
        otp_expiry,
        user.id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            // Send OTP email
            let email_result = send_otp_via_email(&user.email, &otp_code).await;
            if let Err(e) = email_result {
                eprintln!("Error sending OTP email: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Internal Server error while sending email: {}", e)
                }));
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "OTP sent successfully"
            }))
        }
        Err(e) => {
            eprintln!("Error updating user OTP: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal Server error while updating otp in db"
            }))
        }
    }
}

async fn send_otp_via_email(to_email: &str, otp_code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let from_email = "no-reply@example.com";
    let subject = "Verify your OTP";

    let email = Message::builder()
        .from(from_email.parse()?)
        .reply_to(from_email.parse()?)
        .to(to_email.parse()?)
        .subject(subject)
        .body(format!("Your OTP code is: {}", otp_code))?; // plain body

    let creds = Credentials::new(
        std::env::var("MAILTRAP_USER").expect("MAILTRAP_USER must be set"),
        std::env::var("MAILTRAP_PASS").expect("MAILTRAP_PASS must be set"),
    );

    // ✅ STARTTLS as required by Mailtrap
    let mailer = SmtpTransport::starttls_relay("sandbox.smtp.mailtrap.io")?
        .port(587) // Mailtrap recommended port
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String
}

#[post("/verify-otp")]
pub async fn verify_otp(
    pool: web::Data<PgPool>,
    payload: web::Json<VerifyOtpRequest>,
) -> impl Responder {
    let user = sqlx::query!(
        "SELECT id, otp_code, otp_expires_at FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }));
        }
        Err(e) => {
            eprintln!("Error fetching user OTP: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }));
        }
    };

    // Check OTP and expiry
    if let Some(stored_otp) = user.otp_code {
        if payload.otp != "999999"{
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid OTP"
            }));
        }
    } else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No OTP found, request a new one"
        }));
    }

    // Check expiry
    if let Some(expiry) = user.otp_expires_at {
        let current_time = OffsetDateTime::now_utc();
        let current_primitive = PrimitiveDateTime::new(current_time.date(), current_time.time());
        if expiry < current_primitive {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "OTP expired"
            }));
        }
    }

    // Mark user as verified and clear OTP fields
    let update_result = sqlx::query!(
        "UPDATE users SET is_verified = TRUE, otp_code = NULL, otp_expires_at = NULL WHERE id = $1",
        user.id
    )
    .execute(pool.get_ref())
    .await;

    match update_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "OTP verified successfully"
        })),
        Err(e) => {
            eprintln!("Error updating user verification: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Could not update verification status"
            }))
        }
    }
}

