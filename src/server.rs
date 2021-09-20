use std::net::SocketAddr;

use axum::handler::get;
use axum::http::{HeaderValue, Response};
use axum::response::{Html, IntoResponse};
use axum::Router;
use tower::ServiceBuilder;

use crate::wasm_bindgen::WasmBindgenOutput;
use crate::Result;

pub async fn run_server(output: WasmBindgenOutput) -> Result<()> {
    let WasmBindgenOutput { js, wasm } = output;

    let middleware_stack = ServiceBuilder::new().into_inner();

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../static/index.html")) }))
        .route("/wasm.js", get(|| async { WithContentType("application/javascript", js) }))
        .route("/wasm.wasm", get(|| async { WithContentType("application/wasm", wasm) }))
        .layer(middleware_stack);

    let port = 1334;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("starting webserver at {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

struct WithContentType<T>(&'static str, T);
impl<T: IntoResponse> IntoResponse for WithContentType<T> {
    type Body = T::Body;
    type BodyError = T::BodyError;

    fn into_response(self) -> Response<Self::Body> {
        let mut response = self.1.into_response();
        response.headers_mut().insert("Content-Type", HeaderValue::from_static(self.0));
        response
    }
}
