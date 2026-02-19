use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::errors::{VoxError, VoxResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,        // user id
    pub username: String,
    pub exp: usize,       // expiration timestamp
    pub iat: usize,       // issued at
}

/// Hash un mot de passe avec Argon2id
pub fn hash_password(password: &str) -> VoxResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| VoxError::InternalError)?
        .to_string();
    Ok(hash)
}

/// Vérifie un mot de passe contre son hash
pub fn verify_password(password: &str, hash: &str) -> VoxResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| VoxError::InternalError)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Génère un JWT
pub fn create_token(user_id: Uuid, username: &str, secret: &str, expiration_hours: i64) -> VoxResult<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiration_hours);

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| VoxError::InternalError)
}

/// Valide et décode un JWT
pub fn validate_token(token: &str, secret: &str) -> VoxResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| VoxError::Unauthorized)
}
