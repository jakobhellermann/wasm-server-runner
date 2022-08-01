use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::error_handling::HandleError;
use axum::extract::Path;
use axum::headers::HeaderName;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use axum_server_dual_protocol::ServerExt;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::wasm_bindgen::WasmBindgenOutput;
use crate::Result;

fn generate_version() -> String {
    std::iter::repeat_with(fastrand::alphanumeric).take(12).collect()
}

pub struct Options {
    pub title: String,
    pub address: String,
    pub directory: PathBuf,
    pub https: bool,
    pub no_module: bool,
}

pub async fn run_server(options: Options, output: WasmBindgenOutput) -> Result<()> {
    let WasmBindgenOutput { js, wasm, snippets, local_modules } = output;

    let middleware_stack = ServiceBuilder::new()
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cross-origin-opener-policy"),
            HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cross-origin-embedder-policy"),
            HeaderValue::from_static("require-corp"),
        ))
        .into_inner();

    let version = generate_version();

    // Use a custom `index.html` if the user has one in the serving directory.
    let html_source = std::fs::read_to_string(&options.directory.join("index.html"))
        .map(Cow::Owned)
        .unwrap_or_else(|_| Cow::Borrowed(include_str!("../static/index.html")));
    let mut html = html_source.replace("{{ TITLE }}", &options.title);

    if options.no_module {
        html = html
            .replace("{{ NO_MODULE }}", "<script src=\"./api/wasm.js\"></script>")
            .replace("// {{ MODULE }}", "");
    } else {
        html = html
            .replace("// {{ MODULE }}", "import wasm_bindgen from './api/wasm.js';")
            .replace("{{ NO_MODULE }}", "");
    };

    let serve_dir =
        HandleError::new(get_service(ServeDir::new(options.directory)), internal_server_error);

    let app = Router::new()
        .route("/", get(move || async { Html(html) }))
        .route("/api/wasm.js", get(|| async { WithContentType("application/javascript", js) }))
        .route("/api/wasm.wasm", get(|| async { WithContentType("application/wasm", wasm) }))
        .route("/api/version", get(move || async { version }))
        .route(
            "/api/snippets/*rest",
            get(|Path(path): Path<String>| async move {
                match get_snippet_source(&path, &local_modules, &snippets) {
                    Ok(source) => Ok(WithContentType("application/javascript", source)),
                    Err(e) => {
                        tracing::error!("failed to serve snippet `{path}`: {e}");
                        Err(e)
                    }
                }
            }),
        )
        .fallback_service(serve_dir)
        .layer(middleware_stack);

    let mut address_string = options.address;
    if !address_string.contains(':') {
        address_string +=
            &(":".to_owned() + &pick_port::pick_free_port(1334, 10).unwrap_or(1334).to_string());
    }
    let addr: SocketAddr = address_string.parse().expect("Couldn't parse address");

    if options.https {
        let certificate = certificate::certificate()?;
        let config =
            RustlsConfig::from_der(vec![certificate.certificate], certificate.private_key).await?;

        tracing::info!("starting webserver at https://{}", addr);
        axum_server_dual_protocol::bind_dual_protocol(addr, config)
            .set_upgrade(true)
            .serve(app.into_make_service())
            .await?;
    } else {
        tracing::info!("starting webserver at http://{}", addr);
        axum_server::bind(addr).serve(app.into_make_service()).await?;
    }

    Ok(())
}

fn get_snippet_source(
    path: &str,
    local_modules: &HashMap<String, String>,
    snippets: &HashMap<String, Vec<String>>,
) -> Result<String, &'static str> {
    if let Some(module) = local_modules.get(path) {
        return Ok(module.clone());
    };

    let (snippet, inline_snippet_name) = path.split_once('/').ok_or("invalid snippet path")?;
    let index = inline_snippet_name
        .strip_prefix("inline")
        .and_then(|path| path.strip_suffix(".js"))
        .ok_or("invalid snippet name in path")?;
    let index: usize = index.parse().map_err(|_| "invalid index")?;
    let snippet = snippets
        .get(snippet)
        .ok_or("invalid snippet name")?
        .get(index)
        .ok_or("snippet index out of bounds")?;
    Ok(snippet.clone())
}

struct WithContentType<T>(&'static str, T);
impl<T: IntoResponse> IntoResponse for WithContentType<T> {
    fn into_response(self) -> Response {
        let mut response = self.1.into_response();
        response.headers_mut().insert("Content-Type", HeaderValue::from_static(self.0));
        response
    }
}

async fn internal_server_error(error: impl std::fmt::Display) -> impl IntoResponse {
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

mod certificate {
    use std::io::ErrorKind;
    use std::path::Path;

    use crate::Result;
    use directories::ProjectDirs;

    pub struct Certificate {
        pub certificate: Vec<u8>,
        pub private_key: Vec<u8>,
    }

    pub fn certificate() -> Result<Certificate> {
        let directories = match ProjectDirs::from("", "", "wasm-server-runner") {
            Some(directories) => directories,
            None => {
                tracing::warn!("failed to determine application directory");
                return generate();
            }
        };

        let path = directories.data_local_dir();

        let certificate = match read(&path.join("certificate.der")) {
            Ok(Some(certificate)) => certificate,
            Ok(None) => return generate_in(path),
            Err(()) => return generate(),
        };

        let private_key = match read(&path.join("private_key.der")) {
            Ok(Some(private_key)) => private_key,
            Ok(None) => return generate_in(path),
            Err(()) => return generate(),
        };

        tracing::info!("re-using certificate from \"{}\"", path.display());

        Ok(Certificate { certificate, private_key })
    }

    fn read(path: &Path) -> Result<Option<Vec<u8>>, ()> {
        match std::fs::read(path) {
            Ok(file) => Ok(Some(file)),
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    tracing::error!("error reading file from \"{}\": {error}", path.display());
                    Err(())
                }
            }
        }
    }

    fn write(path: &Path, data: &[u8]) -> Result<(), ()> {
        match std::fs::write(path, data) {
            Ok(()) => Ok(()),
            Err(error) => {
                tracing::error!("error saving file to \"{}\": {error}", path.display());
                Err(())
            }
        }
    }

    fn generate() -> Result<Certificate> {
        tracing::warn!("generated temporary certificate");

        generate_internal()
    }

    fn generate_in(path: &Path) -> Result<Certificate> {
        let certificate = generate_internal()?;

        if let Err(error) = std::fs::create_dir_all(path) {
            tracing::error!("error creating directory \"{}\": {error}", path.display());
            tracing::warn!("generated temporary certificate");
            return Ok(certificate);
        }

        if let Err(()) = write(&path.join("certificate.der"), &certificate.certificate)
            .and_then(|_| write(&path.join("private_key.der"), &certificate.private_key))
        {
            tracing::warn!("generated temporary certificate");
            return Ok(certificate);
        }

        tracing::info!("generated new certificate in \"{}\"", path.display());
        Ok(certificate)
    }

    fn generate_internal() -> Result<Certificate> {
        let certificate = rcgen::generate_simple_self_signed([String::from("localhost")])?;

        Ok(Certificate {
            certificate: certificate.serialize_der()?,
            private_key: certificate.serialize_private_key_der(),
        })
    }
}
