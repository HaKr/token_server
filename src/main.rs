#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::expect_used
)]
use std::{fmt::Display, io, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, head, post},
    Router,
};
use axum_server::Handle;
use clap::Parser;
use clap_duration::assign_duration_range_validator;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, enabled, error, info, trace, warn, Level};

use duration_human::{DurationHuman, DurationHumanValidator};

mod token_server;
use token_server::{routes, TokenStore};

assign_duration_range_validator!( TOKEN_LIFETIME_RANGE = {default: 2h, min: 1500ms, max: 60day});
assign_duration_range_validator!( PURGE_INTERVAL_RANGE = {min: 1500ms, default: 1min, max: 90min});

#[derive(Parser)]
struct ServerOptions {
    /// allow for HEAD /dump endpoint to log all metadata
    #[arg(long)]
    dump_enabled: bool,

    /// allow for GET /shutdown endpoint to shutdown this server
    #[arg(long)]
    shutdown_enabled: bool,

    /// Which port to listen on
    #[arg(short, long, default_value_t = 3666, value_parser = clap::value_parser!(u16).range(3000..) ) ]
    port: u16,

    /// What frequency to remove expired tokens
    #[arg(
        long,
        help = format!("What frequency to remove expired tokens, {}", PURGE_INTERVAL_RANGE),
        default_value = PURGE_INTERVAL_RANGE.default,
        value_parser = {|interval: &str|PURGE_INTERVAL_RANGE.parse_and_validate(interval)}
    )]
    purge_interval: DurationHuman,

    /// How long does a token remain valid
    #[arg(
        long,
        help = format!("How long does a token remain valid, {}", TOKEN_LIFETIME_RANGE),
        default_value = TOKEN_LIFETIME_RANGE.default,
        value_parser = {|lifetime: &str|TOKEN_LIFETIME_RANGE.parse_and_validate(lifetime)}
    )]
    token_lifetime: DurationHuman,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let opts = ServerOptions::parse();
    info!("Token server listening: {}", opts);

    let handle = Handle::new();
    let log_debug_enabled = enabled!(Level::DEBUG);
    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let token_store = Arc::new(
        TokenStore::default()
            .with_token_lifetime(opts.token_lifetime)
            .with_handle(handle.clone()),
    );
    let token_store_during_purge = token_store.clone();

    tokio::spawn(async move {
        loop {
            sleep((&opts.purge_interval).into()).await;

            token_store_during_purge
                .clone()
                .remove_expired_tokens()
                .map_or_else(
                    |err| error!("PURGE failed: {}", err),
                    |purged| {
                        if log_debug_enabled && purged.purged > 0 {
                            debug!("{}", purged);
                        } else {
                            trace!("{}", purged);
                        }
                    },
                );
        }
    });

    let mut token_server_routes = Router::new().route(
        "/token",
        post(routes::create_token)
            .put(routes::update_token)
            .delete(routes::remove_token),
    );

    if opts.dump_enabled && log_debug_enabled {
        token_server_routes = token_server_routes.route("/dump", head(routes::dump_meta));
    } else {
        warn!("HEAD /dump will not provide logging; use RUSTLOG='token_server=debug'");
    }

    if opts.shutdown_enabled {
        token_server_routes = token_server_routes.route("/shutdown", get(routes::shutdown_server));
    }

    axum_server::bind(addr)
        .handle(handle)
        .serve(
            token_server_routes
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .with_state(token_store)
                .into_make_service(),
        )
        .await?;

    Ok(())
}

impl Display for ServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[inline]
        fn is_enabled(switch: bool) -> String {
            String::from(if switch { "enabled" } else { "disabled" })
        }

        f.write_fmt(format_args!(
            "Port: {portnumber}, Token lifetime: {lifetime:#}, Purge cycle: {interval:#}, HEAD /dump {dump_enabled}, GET /shutdown {shutdown_enabled}",
            portnumber = self.port,
            lifetime=self.token_lifetime,
            interval=self.purge_interval,
            dump_enabled = is_enabled(self.dump_enabled),
            shutdown_enabled = is_enabled(self.shutdown_enabled)
        ))
    }
}
