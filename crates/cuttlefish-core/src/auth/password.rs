//! Password hashing and verification using Argon2id.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use super::AuthError;

/// Hash a password using Argon2id.
///
/// Uses recommended parameters: memory=65536KB, iterations=3, parallelism=4.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 3, 4, None)
            .map_err(|e| AuthError::HashingFailed(e.to_string()))?,
    );

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::HashingFailed(e.to_string()))?;

    Ok(hash.to_string())
}

/// Verify a password against a stored hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| AuthError::VerificationFailed(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AuthError::VerificationFailed(e.to_string())),
    }
}

/// Password strength requirements.
#[derive(Debug, Clone)]
pub struct PasswordRequirements {
    /// Minimum length (default: 8).
    pub min_length: usize,
    /// Require at least one uppercase letter.
    pub require_uppercase: bool,
    /// Require at least one lowercase letter.
    pub require_lowercase: bool,
    /// Require at least one digit.
    pub require_digit: bool,
}

impl Default for PasswordRequirements {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
        }
    }
}

/// Validate password strength against requirements.
pub fn validate_password_strength(password: &str) -> Result<(), AuthError> {
    let reqs = PasswordRequirements::default();

    if password.len() < reqs.min_length {
        return Err(AuthError::WeakPassword(format!(
            "Password must be at least {} characters",
            reqs.min_length
        )));
    }

    if reqs.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if reqs.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if reqs.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one digit".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "SecurePass123!";
        let hash = hash_password(password).expect("hashing should succeed");

        assert!(hash.starts_with("$argon2id$"));

        let verified = verify_password(password, &hash).expect("verification should succeed");
        assert!(verified);

        let wrong_verified =
            verify_password("WrongPassword", &hash).expect("verification should succeed");
        assert!(!wrong_verified);
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hash1 = hash_password("Password1!").expect("hash1");
        let hash2 = hash_password("Password1!").expect("hash2");

        // Same password should produce different hashes (different salts)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_validate_password_strength_valid() {
        assert!(validate_password_strength("SecurePass123!").is_ok());
        assert!(validate_password_strength("Abcdefg1").is_ok());
        assert!(validate_password_strength("MyP4ssword").is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let result = validate_password_strength("Short1!");
        assert!(result.is_err());
        assert!(matches!(result, Err(AuthError::WeakPassword(_))));
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let result = validate_password_strength("lowercase123");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let result = validate_password_strength("UPPERCASE123");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let result = validate_password_strength("NoDigitsHere!");
        assert!(result.is_err());
    }
}
