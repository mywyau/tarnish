use bcrypt::{hash, verify, DEFAULT_COST};


// Hash a user's password when registering
fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

// Verify a password when logging in
fn verify_password(password: &str, hashed_password: &str) -> Result<bool, bcrypt::BcryptError> {
    let result = verify(password, hashed_password)?;
    Ok(result)
}
