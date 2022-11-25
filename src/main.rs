use std::{fmt::Display, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::Extension,
    routing::{delete, head, post, put},
    Router,
};
use gumdrop::Options;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, enabled, error, info, trace, warn, Level};

mod token_server;
use token_server::{
    format_duration, parse_duration_with_min_and_max, routes, InvalidDuration, TokenServerState,
};

const TOKEN_LIFETIME_MIN_SEC: u64 = 30 * 60;
const TOKEN_LIFETIME_MAX_SEC: u64 = 96 * 60 * 60;
const PURGE_INTERVAL_MIN_SEC: u64 = 1;
const PURGE_INTERVAL_MAX_SEC: u64 = 90 * 60;

#[derive(Options)]
struct ServerOptions {
    #[options(help = "print this help message")]
    help: bool,

    #[options(help = "allow for HEAD /dump endpoint to log all metadata")]
    dump: bool,

    #[options(help = "Which port to listen on", default = "3666")]
    port: u16,

    #[options(
        help = "What frequency to remove expired tokens, between 1s and 90min",
        default = "1min",
        parse(try_from_str = "parse_purge_interval_duration")
    )]
    purge_interval: Duration,

    #[options(
        help = "How long does a token remain valid, between 30min and 96h",
        default = "2h",
        parse(try_from_str = "parse_token_lifetime_duration")
    )]
    token_lifetime: Duration,
}

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    tracing_subscriber::fmt::init();

    let opts = ServerOptions::parse_args_default_or_exit();
    info!("Token server listening: {}", opts);

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let state = Arc::new(TokenServerState::default().with_token_lifetime(opts.token_lifetime));
    let clean_state = state.clone();

    tokio::spawn(async move {
        loop {
            sleep(opts.purge_interval).await;
            match clean_state.clone().remove_expired_tokens() {
                Ok(purged) => {
                    if enabled!(Level::DEBUG) && purged.purged > 0 {
                        debug!("PURGED: {}", purged);
                    } else {
                        trace!("PURGED: {}", purged);
                    }
                }
                Err(e) => error!("PURGE failed: {}", e),
            }
        }
    });

    let mut token_server_routes = Router::new()
        .route("/token", post(routes::create_token))
        .route("/token", put(routes::update_token))
        .route("/token", delete(routes::remove_token));

    if opts.dump {
        if !enabled!(Level::DEBUG) {
            warn!("HEAD /dump will not provide logging; use RUSTLOG='token_server=debug'");
        }
        token_server_routes = token_server_routes.route("/dump", head(routes::dump_meta));
    }

    axum::Server::bind(&addr)
        .serve(
            token_server_routes
                .layer(
                    ServiceBuilder::new()
                        .layer(TraceLayer::new_for_http())
                        .layer(Extension(state)),
                )
                .into_make_service(),
        )
        .await?;

    Ok(())
}

fn parse_token_lifetime_duration(option: &str) -> Result<Duration, InvalidDuration> {
    parse_duration_with_min_and_max(option, TOKEN_LIFETIME_MIN_SEC, TOKEN_LIFETIME_MAX_SEC)
}

fn parse_purge_interval_duration(option: &str) -> Result<Duration, InvalidDuration> {
    parse_duration_with_min_and_max(option, PURGE_INTERVAL_MIN_SEC, PURGE_INTERVAL_MAX_SEC)
}

impl Display for ServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Port: {}, Token lifetime: {}, Purge cycle: {}, HEAD /dump endpoint {}",
            self.port,
            format_duration(self.token_lifetime.as_secs()),
            format_duration(self.purge_interval.as_secs()),
            if self.dump { "enabled" } else { "disabled" }
        ))
    }
}
