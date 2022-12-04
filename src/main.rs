use std::{fmt::Display, net::SocketAddr, sync::Arc};

use axum::{
    routing::{delete, head, post, put},
    Router,
};
use clap::Parser;
use clap_duration::assign_duration_range_validator;
use duration_human::{DurationHuman, DurationHumanValidator};
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, enabled, error, info, trace, warn, Level};

mod token_server;
use token_server::{routes, TokenServerState};

assign_duration_range_validator!( TOKEN_LIFETIME_RANGE = {default: 2h, min: 10min, max: 60day});
assign_duration_range_validator!( PURGE_INTERVAL_RANGE = {min: 1500ms, default: 1min, max: 90min});

#[derive(Parser)]
struct ServerOptions {
    /// allow for HEAD /dump endpoint to log all metadata
    #[arg(long)]
    dump: bool,

    /// Which port to listen on
    #[arg(short, long, default_value_t = 3666, value_parser = clap::value_parser!(u16).range(3000..) ) ]
    port: u16,

    #[arg(
        long,
        help = format!("What frequency to remove expired tokens, between {}", PURGE_INTERVAL_RANGE),
        default_value = PURGE_INTERVAL_RANGE.default,
        value_parser = {|interval: &str|PURGE_INTERVAL_RANGE.parse_and_validate(interval)}
    )]
    purge_interval: DurationHuman,

    #[arg(
        long,
        help = format!("How long does a token remain valid, between {}", TOKEN_LIFETIME_RANGE),
        default_value = TOKEN_LIFETIME_RANGE.default,
        value_parser = {|lifetime: &str|TOKEN_LIFETIME_RANGE.parse_and_validate(lifetime)}
    )]
    token_lifetime: DurationHuman,
}

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    tracing_subscriber::fmt::init();

    let opts = ServerOptions::parse();
    info!("Token server listening: {}", opts);

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let state = Arc::new(TokenServerState::default().with_token_lifetime(opts.token_lifetime));
    let clean_state = state.clone();
    let log_debug_enabled = enabled!(Level::DEBUG);

    tokio::spawn(async move {
        loop {
            sleep((&opts.purge_interval).into()).await;
            match clean_state.clone().remove_expired_tokens() {
                Ok(purged) => {
                    if log_debug_enabled && purged.purged > 0 {
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

    if opts.dump && log_debug_enabled {
        token_server_routes = token_server_routes.route("/dump", head(routes::dump_meta));
    } else {
        warn!("HEAD /dump will not provide logging; use RUSTLOG='token_server=debug'");
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

impl Display for ServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Port: {}, Token lifetime: {:#}, Purge cycle: {:#}, HEAD /dump endpoint {}",
            self.port,
            self.token_lifetime,
            self.purge_interval,
            if self.dump { "enabled" } else { "disabled" }
        ))
    }
}
