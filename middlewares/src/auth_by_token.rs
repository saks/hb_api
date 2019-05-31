use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorUnauthorized, ParseError},
    http::{header, HeaderValue},
    HttpMessage,
};
use futures::future::{ok, Future, FutureResult};
use futures::Poll;
use octo_budget_lib::auth_token::{AuthToken, UserId};

// There are two step in middleware processing.
// 1. Middleware initialization, middleware factory get called with
//    next service in chain as parameter.
// 2. Middleware's call method get called with normal request.

pub struct AuthByToken {
    secret: &'static [u8],
}

impl AuthByToken {
    pub fn new(secret: &'static str) -> Self {
        Self {
            secret: secret.as_bytes(),
        }
    }
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for AuthByToken
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
    S::Error: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Transform = AuthByTokenMiddleware<S>;
    type InitError = ();
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthByTokenMiddleware {
            service,
            secret: self.secret,
        })
    }
}

pub struct AuthByTokenMiddleware<S> {
    service: S,
    secret: &'static [u8],
}

impl<S> AuthByTokenMiddleware<S> {
    fn authenticate(&self, req: &mut ServiceRequest) -> actix_web::Result<()> {
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .ok_or_else(|| ErrorUnauthorized(ParseError::Header))?
            .to_str()
            .map_err(ErrorUnauthorized)?;

        let mut parts = auth_header.split_whitespace();

        if let Some(token_type) = parts.next() {
            if token_type != "JWT" {
                return Err(ErrorUnauthorized("Wrong token type!"));
            }
        }

        match parts.next() {
            Some(token) => AuthToken::from(token, self.secret)
                .map(|auth_token| {
                    req.extensions_mut().insert(auth_token.user_id());
                    ()
                })
                .map_err(|_| ErrorUnauthorized("TODO: bad token error")),
            None => Err(ErrorUnauthorized("Wrong token format!")),
        }
    }
}

impl<S, B> Service for AuthByTokenMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
    S::Error: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        if let Err(e) = self.authenticate(&mut req) {
            return Box::new(ok(req.error_response(e)));
        }

        Box::new(self.service.call(req).and_then(|res| Ok(res)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{call_service, init_service, TestRequest};
    use actix_web::{web, App, HttpResponse};

    #[test]
    fn authenticated() {
        let secret = "foo";
        let user_id = 123;
        let token = AuthToken::new(user_id).encrypt(secret.as_bytes());

        let mut app = init_service(
            App::new()
                .wrap(AuthByToken::new(secret))
                .service(web::resource("/v1/something/").to(|| HttpResponse::Ok())),
        );
    }
}
