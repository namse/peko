use color_eyre::{eyre::eyre, Result};
use http_body_util::BodyExt;
use hyper::{server::conn::http1, Request, StatusCode};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::config::Config;

pub async fn execute(port: Option<u16>, static_dir: Option<PathBuf>) -> Result<()> {
    println!("Starting local fn0 server...\n");

    let config = Config::load("fn0.toml").map_err(|e| {
        eyre!(
            "Failed to load fn0.toml: {}. Make sure you're in a fn0 project directory.",
            e
        )
    })?;

    crate::commands::build::execute().await?;

    let wasm_path = PathBuf::from(match config.language_env {
        crate::config::LanguageEnvironment::TypescriptBunHono => "./dist/component.wasm",
        crate::config::LanguageEnvironment::TypescriptBunAstro => "./dist/server/component.wasm",
    });
    let cwasm_path = wasm_path.with_extension("cwasm");

    let cwasm = if !cwasm_path.exists() {
        println!("\nStart compile wasm to cwasm...\n");
        let wasm = fs::read(wasm_path).await?;
        let cwasm = fn0::compile::compile(&wasm).map_err(|e| eyre!(e))?;
        fs::write(cwasm_path, &cwasm).await?;
        println!("\nFinish compile wasm to cwasm...\n");
        cwasm
    } else {
        fs::read(cwasm_path).await?
    };

    println!("\nServer starting...\n");

    let listener = open_tcp_listener(port.unwrap_or(3000), port.is_none())?;

    println!(
        "Server on http://localhost:{}",
        listener.local_addr().unwrap().port()
    );

    let fn0_one = fn0::one::Fn0One::new(fn0::one::Config { cwasm: &cwasm })
        .await
        .map_err(|e| eyre!(e))?;

    let static_service = static_dir.as_ref().map(ServeDir::new);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let tower_service = ServiceBuilder::new().service(tower::util::service_fn({
            let fn0_one = fn0_one.clone();
            let static_service = static_service.clone();

            move |req: Request<hyper::body::Incoming>| {
                let fn0_one = fn0_one.clone();
                let static_service = static_service.clone();

                async move {
                    let path = req.uri().path();
                    let is_static_asset = static_service.is_some()
                        && path.contains('.')
                        && !path.ends_with(".html")
                        && req.method() == hyper::Method::GET;

                    if !is_static_asset {
                        return fn0_one.run(req).await;
                    }

                    let Some(service) = static_service else {
                        let body = fn0::Full::new(fn0::Bytes::from("Not Found"))
                            .map_err(|_| fn0::ErrorCode::InternalError(None));
                        let wrapped = fn0::HyperOutgoingBody::new(body);
                        let mut res = hyper::Response::new(wrapped);
                        *res.status_mut() = StatusCode::NOT_FOUND;
                        return Ok(res);
                    };

                    match service.oneshot(req).await {
                        Ok(res) if res.status() != StatusCode::NOT_FOUND => {
                            let (res_parts, res_body) = res.into_parts();

                            match res_body.collect().await {
                                Ok(collected) => {
                                    let bytes = collected.to_bytes();
                                    let body = fn0::Full::new(bytes)
                                        .map_err(|_| fn0::ErrorCode::InternalError(None));
                                    let wrapped = fn0::HyperOutgoingBody::new(body);
                                    Ok(hyper::Response::from_parts(res_parts, wrapped))
                                }
                                Err(_) => {
                                    let body =
                                        fn0::Full::new(fn0::Bytes::from("Error reading file"))
                                            .map_err(|_| fn0::ErrorCode::InternalError(None));
                                    let wrapped = fn0::HyperOutgoingBody::new(body);
                                    let mut res = hyper::Response::new(wrapped);
                                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                    Ok(res)
                                }
                            }
                        }
                        _ => {
                            let body = fn0::Full::new(fn0::Bytes::from("Not Found"))
                                .map_err(|_| fn0::ErrorCode::InternalError(None));
                            let wrapped = fn0::HyperOutgoingBody::new(body);
                            let mut res = hyper::Response::new(wrapped);
                            *res.status_mut() = StatusCode::NOT_FOUND;
                            Ok(res)
                        }
                    }
                }
            }
        }));

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    TowerToHyperService::new(tower_service),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn open_tcp_listener(mut port: u16, increment_on_fail: bool) -> Result<TcpListener> {
    loop {
        let socket = try_open_tcp_listener(port);
        if !increment_on_fail || socket.is_ok() {
            return socket.map_err(Into::into);
        }
        if port == u16::MAX {
            return Err(eyre!("Failed to open socket"));
        }
        port += 1;
    }
}

fn try_open_tcp_listener(port: u16) -> std::io::Result<TcpListener> {
    let addr: SocketAddr = format!("[::]:{port}").parse().unwrap();
    let domain = Domain::for_address(addr);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_only_v6(false)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;

    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(std_listener)?;

    Ok(listener)
}
