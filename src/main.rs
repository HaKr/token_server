use std::{fmt::Display, net::SocketAddr, sync::Arc};

use axum::{
    routing::{delete, head, post, put},
    Router,
};
use clap::Parser;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, enabled, error, info, trace, warn, Level};

mod token_server;
use token_server::{routes, Duration, DurationRange, InvalidDuration, TokenServerState};

const TOKEN_LIFETIME_RANGE: DurationRange = DurationRange::new(30 * 60, 96 * 60 * 60);
const PURGE_INTERVAL_RANGE: DurationRange = DurationRange::new(1, 90 * 60);

const TOKEN_LIFETIME_DEFAULT: Duration = Duration::new(60);
const PURGE_INTERVAL_DEFAULT: Duration = Duration::new(2 * 60 * 60);

#[derive(Parser)]
struct ServerOptions {
    /// allow for HEAD /dump endpoint to log all metadata
    #[arg(short, long)]
    dump: bool,

    /// Which port to listen on
    #[arg(short = 'P', long, default_value_t = 3666)]
    port: u16,

    /// What frequency to remove expired tokens, between 1s and 90min
    #[arg(
        short, long,
        default_value = TOKEN_LIFETIME_DEFAULT,
        value_parser = parse_purge_interval_duration
    )]
    purge_interval: Duration,

    // How long does a token remain valid, between 30min and 96h
    #[arg(
        short, long,
        default_value = PURGE_INTERVAL_DEFAULT,
        value_parser =parse_token_lifetime_duration
    )]
    token_lifetime: Duration,
}

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    tracing_subscriber::fmt::init();

    let opts = ServerOptions::parse();
    info!("Token server listening: {}", opts);

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let state = Arc::new(TokenServerState::default().with_token_lifetime(opts.token_lifetime));
    let clean_state = state.clone();

    tokio::spawn(async move {
        loop {
            sleep((&opts.purge_interval).into()).await;
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
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .with_state(state)
                .into_make_service(),
        )
        .await?;

    Ok(())
}

fn parse_token_lifetime_duration(option: &str) -> Result<Duration, InvalidDuration> {
    TOKEN_LIFETIME_RANGE.contains(option.try_into()?)
}

fn parse_purge_interval_duration(option: &str) -> Result<Duration, InvalidDuration> {
    PURGE_INTERVAL_RANGE.contains(option.try_into()?)
}

impl Display for ServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Port: {}, Token lifetime: {}, Purge cycle: {}, HEAD /dump endpoint {}",
            self.port,
            self.token_lifetime,
            self.purge_interval,
            if self.dump { "enabled" } else { "disabled" }
        ))
    }
}
