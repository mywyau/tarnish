use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};


mod jwt_token_service {
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String, // subject (e.g., user id or username)
        exp: usize,  // expiration timestamp
        role: String, // user role (admin, editor, viewer)
    }

    fn generate_jwt(username: &str, role: &str) -> String {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() + 60 * 60; // Token expires in 1 hour

        let claims = Claims {
            sub: username.to_owned(),
            exp: expiration as usize,
            role: role.to_owned(),
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();
        token
    }

    fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret("secret".as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}