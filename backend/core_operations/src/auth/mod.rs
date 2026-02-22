// Authentication service using Argon2id
// Integrates with existing JWT system in GraphQL service

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Password hashing failed: {0}")]
    HashingError(String),
    
    #[error("Password verification failed")]
    VerificationFailed,
    
    #[error("Invalid password hash format")]
    InvalidHashFormat,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Account is inactive or suspended")]
    AccountInactive,
    
    #[error("Too many failed login attempts")]
    AccountLocked,
}

/// Hash a password using Argon2id
/// 
/// Example:
/// ```
/// let password = "my_secure_password";
/// let hash = hash_password(password)?;
/// // Store `hash` in database (Users.password_hash column)
/// ```
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::HashingError(e.to_string()))
}

/// Verify a password against a stored hash
/// 
/// Example:
/// ```
/// let password = "user_input_password";
/// let stored_hash = user.password_hash; // From database
/// 
/// if verify_password(password, &stored_hash)? {
///     // Password correct
/// } else {
///     // Password incorrect
/// }
/// ```
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AuthError::InvalidHashFormat)?;
    
    let argon2 = Argon2::default();
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// User status from database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

impl From<String> for UserStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            "suspended" => UserStatus::Suspended,
            _ => UserStatus::Inactive,
        }
    }
}

/// Login credentials
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Registration data
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
    pub phone: Option<String>,
}

/// User data after authentication
#[derive(Debug, Serialize)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub status: UserStatus,
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        return Err(AuthError::HashingError(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    
    // Add more checks as needed (uppercase, numbers, special chars, etc.)
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "my_secure_password_123";
        
        // Hash
        let hash = hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
        
        // Verify correct password
        assert!(verify_password(password, &hash).unwrap());
        
        // Verify incorrect password
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
    
    #[test]
    fn test_password_validation() {
        assert!(validate_password_strength("short").is_err());
        assert!(validate_password_strength("long_enough_password").is_ok());
    }
    
    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        
        // Hashes should be different (different salts)
        assert_ne!(hash1, hash2);
        
        // But both should verify
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
