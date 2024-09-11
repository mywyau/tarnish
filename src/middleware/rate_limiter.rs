use actix_web::body::{BoxBody, EitherBody};
use actix_web::web;
use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpResponse};
use futures::future::{ok, LocalBoxFuture, Ready};
use redis::AsyncCommands;
use std::task::{Context, Poll};

use actix_web::http::StatusCode;
use serde_json::json;

// Define the `RateLimiter` struct
pub struct RateLimiter {
    redis_client: web::Data<redis::Client>,
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(redis_client: web::Data<redis::Client>, max_requests: u32, window_seconds: u64) -> Self {
        RateLimiter {
            redis_client,
            max_requests,
            window_seconds,
        }
    }
}

// Implement the `Transform` trait for `RateLimiter`
impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddleware {
            service,
            redis_client: self.redis_client.clone(),
            max_requests: self.max_requests,
            window_seconds: self.window_seconds,
        })
    }
}

// Middleware struct holding service and rate limiter configuration
pub struct RateLimiterMiddleware<S> {
    service: S,
    redis_client: web::Data<redis::Client>,
    max_requests: u32,
    window_seconds: u64,
}

// Implement the `Service` trait for `RateLimiterMiddleware`
impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    // In the `call` function
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let redis_client = self.redis_client.clone();
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;

        // Extract the client IP before moving `req`
        let connection_info = req.connection_info().clone();  // Clone connection_info to extract IP
        let client_ip = extract_client_ip(&connection_info);

        // Break `req` into parts before it's moved into the future chain
        let (req_parts, req_body) = req.into_parts();  // Break the request into parts

        // Call the next service in the middleware chain with `req_body`
        let fut = self.service.call(ServiceRequest::from_parts(req_parts.clone(), req_body));

        Box::pin(async move {
            // Handle rate limiting
            match handle_rate_limit(redis_client, &client_ip, max_requests, window_seconds).await {
                Ok(true) => {
                    // Return a custom JSON response for too many requests
                    let custom_response = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                        .json(json!({
                        "error": "Too many requests",
                        "message": "You have exceeded the rate limit. Please try again later.",
                        "retry_after_seconds": window_seconds,
                    }))
                        .map_into_right_body(); // Mapping the body to the correct type

                    return Ok(ServiceResponse::new(req_parts, custom_response));
                }
                Ok(false) => {
                    // Proceed with the request if under rate limit
                    fut.await.map(|res| res.map_into_left_body())
                }
                Err(e) => Err(e),
            }
        })
    }

}

// Helper function to extract the client IP
fn extract_client_ip(connection_info: &actix_web::dev::ConnectionInfo) -> String {
    connection_info.realip_remote_addr().unwrap_or("unknown").to_string()
}

async fn handle_rate_limit(
    redis_client: web::Data<redis::Client>,
    client_ip: &str,
    max_requests: u32,
    window_seconds: u64,
) -> Result<bool, Error> {
    let mut conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Redis connection error")),
    };

    let key = format!("rate_limit:{}", client_ip);

    // Increment the request count for this IP
    let request_count: u32 = match conn.incr(&key, 1).await {
        Ok(count) => count,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Redis increment error")),
    };

    // Set expiration if this is the first request
    if request_count == 1 {
        if let Err(_) = conn.expire::<_, ()>(&key, window_seconds as i64).await {
            return Err(actix_web::error::ErrorInternalServerError("Redis expiration error"));
        }
    }

    // Check if request count exceeds the max allowed
    if request_count > max_requests {
        return Ok(true); // Exceeded rate limit
    }

    Ok(false) // Under the rate limit
}
