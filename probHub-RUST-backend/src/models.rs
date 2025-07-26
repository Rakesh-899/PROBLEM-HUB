// use serde::Deserialize;
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use rand_core::OsRng;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};
use jsonwebtoken::{DecodingKey, Validation, decode};
use jsonwebtoken::errors::Error as JwtError;

pub fn verify_jwt(token: &str) -> Result<Claims, JwtError> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}


pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // subject (we’ll store user id or email)
    pub exp: usize,    // expiration as timestamp
}

pub fn generate_jwt(user_id: i32) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("failed to generate token")
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub new_password: String,
}



