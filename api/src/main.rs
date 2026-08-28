use std::{env, net::SocketAddr};

use intake_desk_api::{app, build_sha};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("intake_desk_api=info,tower_http=info")),
        )
        .init();

    let (port, port_source) = read_port();
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_owned());
    let static_source = if env::var_os("STATIC_DIR").is_some() {
        "supplied"
    } else {
        "default"
    };
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let data_source = if env::var_os("DATA_DIR").is_some() {
        "supplied"
    } else {
        "generated-default"
    };

    info!(
        %address,
        build_sha = build_sha(),
        port_config = port_source,
        static_dir_config = static_source,
        data_dir_config = data_source,
        "starting Intake Desk service"
    );

    let listener = TcpListener::bind(address)
        .await
        .unwrap_or_else(|error| panic!("failed to bind {address}: {error}"));

    axum::serve(
        listener,
        app(static_dir).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("server stopped unexpectedly");
}

fn read_port() -> (u16, &'static str) {
    match env::var("PORT") {
        Ok(value) => match value.parse::<u16>() {
            Ok(port) => (port, "supplied"),
            Err(_) => {
                warn!("PORT was invalid; using 8080");
                (8080, "invalid-fallback")
            }
        },
        Err(_) => (8080, "default"),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}
