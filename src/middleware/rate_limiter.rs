use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error, HttpResponse};
use futures::future::{ok, LocalBoxFuture, Ready};
use redis::AsyncCommands;
use serde_json::json;
use std::task::{Context, Poll};
use actix_web::http::StatusCode;

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
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let redis_client = self.redis_client.clone();
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;

        // Extract the client IP before moving `req`
        let client_ip = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();

        // Call the next service in the chain
        let fut = self.service.call(req); // Moving req here

        Box::pin(async move {
            // Connect to Redis and handle rate limiting
            let mut conn = match redis_client.get_multiplexed_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    log::error!("Failed to connect to Redis: {:?}", e);
                    return Err(actix_web::error::ErrorInternalServerError("Redis connection error"));
                }
            };

            let key = format!("rate_limit:{}", client_ip);

            // Increment the request count for this IP
            let request_count: u32 = match conn.incr(&key, 1).await {
                Ok(count) => count,
                Err(e) => {
                    log::error!("Failed to increment request count in Redis: {:?}", e);
                    return Err(actix_web::error::ErrorInternalServerError("Redis increment error"));
                }
            };

            // Set expiration if this is the first request
            if request_count == 1 {
                if let Err(e) = conn.expire::<_, ()>(&key, window_seconds as i64).await {
                    log::error!("Failed to set expiration for key {}: {:?}", key, e);
                    return Err(actix_web::error::ErrorInternalServerError("Redis expiration error"));
                }
            }

            // Check if the request count exceeds the max allowed
            if request_count > max_requests {
                let custom_response = HttpResponse::TooManyRequests()
                    .json(json!({
                    "error": "Too many requests",
                    "message": "You have exceeded the rate limit. Please try again later.",
                    "retry_after_seconds": window_seconds,
                }))
                    .map_into_right_body();

                return Ok(ServiceResponse::new(fut.await?.into_parts().0, custom_response));
            }

            // Proceed with the next service call in the chain
            fut.await.map(|res| res.map_into_left_body())
        })
    }

}
