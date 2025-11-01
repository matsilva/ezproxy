// Simple HTTP proxy with auth validation using hyper and tower
//
// This server validates the "Authorization" header against a token set in the
// `AUTH_TOKEN` environment variable. If the header is missing or does not match,
// it returns a 401 Unauthorized response. Otherwise, it forwards the request
// to an upstream server defined by the `UPSTREAM_URL` environment variable.
//
// The implementation uses Hyper's client and server APIs together with Tower's
// Service traits for clean separation of concerns.

use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::client::HttpConnector;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use http::header::AUTHORIZATION;

// Simple auth middleware – checks the Authorization header against a token.
async fn authorize(req: Request<Body>, auth_token: String) -> Result<Request<Body>, Response<Body>> {
    // Extract the header value
    match req.headers().get(AUTHORIZATION) {
        Some(value) => {
            if value.to_str().ok() == Some(&auth_token) {
                Ok(req)
            } else {
                Err(Response::builder()
                    .status(401)
                    .body(Body::from("Invalid auth token"))
                    .unwrap())
            }
        }
        None => Err(Response::builder()
            .status(401)
            .body(Body::from("Missing Authorization header"))
            .unwrap()),
    }
}

// Forward the request to the upstream server.
async fn forward(req: Request<Body>, upstream_base: Uri) -> Result<Response<Body>, hyper::Error> {
    // Build new URI preserving path and query.
    let mut parts = upstream_base.into_parts();
    let orig_uri = req.uri();
    // Replace the path and query with those from the original request.
    let path_and_query = orig_uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let new_path = format!("{}", path_and_query);
    parts.path_and_query = Some(new_path.parse().unwrap());
    let uri = Uri::from_parts(parts).expect("valid upstream URI");

    // Clone the request method and headers.
    let (mut parts_req, body) = req.into_parts();
    parts_req.uri = uri;
    // Optionally adjust Host header to match upstream host.
    if let Some(authority) = parts.authority {
        parts_req.headers.insert("host", authority.as_str().parse().unwrap());
    }
    let new_req = Request::from_parts(parts_req, body);

    // Use a Hyper client to send the request.
    let client: Client<HttpConnector> = Client::new();
    client.request(new_req).await
}

async fn handle(req: Request<Body>, auth_token: String, upstream_base: Uri) -> Result<Response<Body>, Infallible> {
    // First, run the auth check.
    match authorize(req, auth_token).await {
        Ok(authenticated_req) => {
            // Forward the request; any client error becomes a 502 response.
            match forward(authenticated_req, upstream_base).await {
                Ok(resp) => Ok(resp),
                Err(_) => Ok(Response::builder()
                    .status(502)
                    .body(Body::from("Bad Gateway"))
                    .unwrap()),
            }
        }
        Err(auth_resp) => Ok(auth_resp),
    }
}

#[tokio::main]
async fn main() {
    // Load configuration from environment variables.
    let auth_token = env::var("AUTH_TOKEN").expect("AUTH_TOKEN must be set");
    let upstream_str = env::var("UPSTREAM_URL").expect("UPSTREAM_URL must be set");
    let upstream_base: Uri = upstream_str.parse().expect("Invalid UPSTREAM_URL");

    // Server address – default to 127.0.0.1:3000 if not provided.
    let addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()
        .expect("Invalid bind address");

    // Build a service that clones the needed config for each request.
    let make_svc = make_service_fn(move |_conn| {
        let auth_token = auth_token.clone();
        let upstream_base = upstream_base.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let auth_token = auth_token.clone();
                let upstream_base = upstream_base.clone();
                handle(req, auth_token, upstream_base)
            }))
        }
    });

    // Build server with Tower middleware (currently only ServiceBuilder placeholder).
    let service = ServiceBuilder::new().service(make_svc);

    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
