use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::request::Parts,
};
use std::{convert::Infallible, net::SocketAddr};

/// Client identity for rate limiting: first `X-Real-IP` value (set by the
/// nginx in front, the app listens on loopback only), else the peer address.
pub struct ClientIp(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Infallible> {
        let header = parts
            .headers
            .get("x-real-ip")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|v| !v.is_empty());
        let ip = match header {
            Some(v) => v.to_owned(),
            None => parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map_or_else(|| "unknown".to_owned(), |c| c.0.ip().to_string()),
        };
        Ok(ClientIp(ip))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    async fn ip(req: Request<()>) -> String {
        let (mut parts, _) = req.into_parts();
        ClientIp::from_request_parts(&mut parts, &())
            .await
            .unwrap()
            .0
    }

    #[tokio::test]
    async fn header_then_connect_info_then_unknown() {
        let r = Request::get("/").header("x-real-ip", "9.9.9.9, 8.8.8.8");
        assert_eq!(ip(r.body(()).unwrap()).await, "9.9.9.9");
        let mut r = Request::get("/").body(()).unwrap();
        r.extensions_mut()
            .insert(ConnectInfo("7.7.7.7:1234".parse::<SocketAddr>().unwrap()));
        assert_eq!(ip(r).await, "7.7.7.7");
        assert_eq!(ip(Request::get("/").body(()).unwrap()).await, "unknown");
    }
}
