use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
    body::MessageBody,
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::rc::Rc;
use crate::models::verify_jwt;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        // Check Authorization header
        let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
        let valid = if let Some(header_val) = auth_header {
            if let Some(token) = header_val.strip_prefix("Bearer ") {
                verify_jwt(token).is_ok()
            } else {
                false
            }
        } else {
            false
        };

        Box::pin(async move {
            if valid {
                let res = srv.call(req).await?;
                Ok(res.map_into_boxed_body())
            } else {
                let (req, _pl) = req.into_parts();
                let res = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized"
                }));
                Ok(ServiceResponse::new(req, res.map_into_boxed_body()))
            }
        })
    }
}
