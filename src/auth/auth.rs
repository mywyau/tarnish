mod auth {
    fn authorize_admin(token: &str) -> bool {
        if let Ok(claims) = verify_jwt(token) {
            return claims.role == "admin";
        }
        false
    }

    fn authorize_editor_or_admin(token: &str) -> bool {
        if let Ok(claims) = verify_jwt(token) {
            return claims.role == "admin" || claims.role == "editor";
        }
        false
    }
}