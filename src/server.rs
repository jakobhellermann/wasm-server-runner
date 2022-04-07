use std::net::SocketAddr;

use axum::http::{HeaderValue, StatusCode};
use axum::response::{Headers, Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::Router;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

use crate::wasm_bindgen::WasmBindgenOutput;
use crate::Result;

fn generate_version() -> String {
    std::iter::repeat_with(fastrand::alphanumeric).take(12).collect()
}

pub struct Options {
    pub title: String,
    pub address: String,
}

pub async fn run_server(options: Options, output: WasmBindgenOutput) -> Result<()> {
    let WasmBindgenOutput { js, compressed_wasm } = output;

    let middleware_stack = ServiceBuilder::new().into_inner();

    let version = generate_version();

    let html = include_str!("../static/index.html").replace("{{ TITLE }}", &options.title);

    let serve_dir = get_service(ServeDir::new(".")).handle_error(internal_server_error);

    let serve_wasm = || async move {
        let headers = Headers(std::iter::once(("Content-Encoding", "gzip")));
        let response = WithContentType("application/wasm", compressed_wasm);
        (headers, response)
    };

    let app = Router::new()
        .route("/", get(move || async { Html(html) }))
        .route("/api/wasm.js", get(|| async { WithContentType("application/javascript", js) }))
        .route("/api/wasm.wasm", get(serve_wasm))
        .route("/api/version", get(move || async { version }))
        .fallback(serve_dir)
        .layer(middleware_stack);

    let mut address_string = options.address;
    if !address_string.contains(":") {
        address_string += &(":".to_owned() + &pick_port::pick_free_port(1334, 10).unwrap_or(1334).to_string());
    }
    let addr: SocketAddr = address_string.parse().expect("Couldn't parse address");

    tracing::info!("starting webserver at http://{}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

struct WithContentType<T>(&'static str, T);
impl<T: IntoResponse> IntoResponse for WithContentType<T> {
    fn into_response(self) -> Response {
        let mut response = self.1.into_response();
        response.headers_mut().insert("Content-Type", HeaderValue::from_static(self.0));
        response
    }
}

async fn internal_server_error(error: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Unhandled internal error: {}", error))
}

mod pick_port {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener, ToSocketAddrs};

    fn test_bind_tcp<A: ToSocketAddrs>(addr: A) -> Option<u16> {
        Some(TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
    }
    fn is_free_tcp(port: u16) -> bool {
        let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
        let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

        test_bind_tcp(ipv6).is_some() && test_bind_tcp(ipv4).is_some()
    }

    fn ask_free_tcp_port() -> Option<u16> {
        let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
        let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0);
        test_bind_tcp(ipv6).or_else(|| test_bind_tcp(ipv4))
    }

    pub fn pick_free_port(starting_at: u16, try_consecutive: u16) -> Option<u16> {
        (starting_at..=starting_at + try_consecutive)
            .find(|&port| is_free_tcp(port))
            .or_else(ask_free_tcp_port)
    }
}
