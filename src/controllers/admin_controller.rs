#[get("/admin")]
async fn admin_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        let token = auth_header.to_str().unwrap().trim_start_matches("Bearer ");
        let claims = verify_jwt(token)?;

        // Check if the user is an admin
        if claims.role == "admin" {
            return Ok(HttpResponse::Ok().body("Welcome, Admin!"));
        } else {
            return Ok(HttpResponse::Forbidden().body("You do not have access to this resource."));
        }
    }
    Err(HttpResponse::Unauthorized().body("Unauthorized").into())
}