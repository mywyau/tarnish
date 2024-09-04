mod auth_controller {

    #[post("/register")]
    async fn register_user(username: String, password: String, email: String, role: String) {
        let hashed_password = hash_password(&password).unwrap();
        // Insert user into the database
        // Set default role to "viewer" or allow specific role assignment
    }


    #[post("/login")]
    async fn login(username: String, password: String) -> Result<String, &'static str> {
        // Find user by username
        let user = get_user_by_username(&username);

        if let Some(user) = user {
            if verify_password(&password, &user.password_hash).unwrap() {
                let token = generate_jwt(&user.username, &user.role);
                Ok(token) // Return JWT token
            } else {
                Err("Invalid credentials")
            }
        } else {
            Err("User not found")
        }
    }

    #[post("/posts")]
    async fn create_post(token: String, post_content: String) -> Result<(), &'static str> {
        if authorize_editor_or_admin(&token) {
            // Allow the user to create a post
        } else {
            Err("Unauthorized")
        }
    }
}