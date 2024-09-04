

#[get("/user")]
async fn user_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        let token = auth_header.to_str().unwrap().trim_start_matches("Bearer ");
        let claims = verify_jwt(token)?;

        // Normal user can access this route
        return Ok(HttpResponse::Ok().body(format!("Hello, {}!", claims.sub)));
    }
    Err(HttpResponse::Unauthorized().body("Unauthorized").into())
}
