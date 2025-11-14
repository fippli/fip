use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type HyperResponse = Response<Full<Bytes>>;

async fn handle_request(req: Request<hyper::body::Incoming>) -> Result<HyperResponse, BoxError> {
    let method = req.method();
    let path = req.uri().path();
    let user_agent = req.headers().get("user-agent").cloned();

    // Echo request info in response
    let mut response_body = serde_json::json!({
        "method": method.as_str(),
        "path": path,
        "status": 200,
    });

    // Handle different paths
    match path {
        "/health" => {
            response_body["message"] = serde_json::json!("OK");
        }
        "/echo" => {
            // Read body if present
            let body_bytes = req.collect().await?.to_bytes();
            if !body_bytes.is_empty() {
                response_body["body"] = serde_json::json!(String::from_utf8_lossy(&body_bytes));
            }
        }
        "/delay" => {
            // Simulate delay
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            response_body["message"] = serde_json::json!("delayed response");
        }
        "/error" => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(r#"{"error": "test error"}"#)))
                .unwrap());
        }
        _ => {
            response_body["message"] = serde_json::json!("not found");
        }
    }

    // Add headers to response
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json");

    // Echo some request headers
    if let Some(ua) = user_agent {
        response = response.header("x-echo-user-agent", ua);
    }

    let json_body = serde_json::to_string(&response_body)?;
    Ok(response.body(Full::new(Bytes::from(json_body))).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    println!("Test server listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
