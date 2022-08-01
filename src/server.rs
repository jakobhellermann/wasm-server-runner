use std::net::SocketAddr;

use axum::headers::ContentEncoding;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{Router, TypedHeader};
use axum_server::tls_rustls::RustlsConfig;
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
        (TypedHeader(ContentEncoding::gzip()), WithContentType("application/wasm", compressed_wasm))
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
        address_string +=
            &(":".to_owned() + &pick_port::pick_free_port(1334, 10).unwrap_or(1334).to_string());
    }
    let addr: SocketAddr = address_string.parse().expect("Couldn't parse address");

    if std::env::var("WASM_SERVER_RUNNER_HTTPS").unwrap_or_else(|_| String::from("0")) == "1" {
        let certificate = rcgen::generate_simple_self_signed([String::from("localhost")])?;
        let config = RustlsConfig::from_der(
            vec![certificate.serialize_der()?],
            certificate.serialize_private_key_der(),
        )
        .await?;

        tracing::info!("starting webserver at https://{}", addr);
        axum_server::bind_rustls(addr, config).serve(app.into_make_service()).await?;
    } else {
        tracing::info!("starting webserver at http://{}", addr);
        axum_server::bind(addr).serve(app.into_make_service()).await?;
    }

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
