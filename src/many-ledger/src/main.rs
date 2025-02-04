use clap::Parser;
use many::server::module::{abci_backend, ledger, ledger_transactions};
use many::server::ManyServer;
use many::transport::http::HttpServer;
use many::types::identity::cose::CoseKeyIdentity;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::debug;
use tracing::level_filters::LevelFilter;

mod error;
mod module;
mod storage;

use module::*;

#[derive(Parser, Debug)]
struct Opts {
    /// Increase output logging verbosity to DEBUG level.
    #[clap(short, long, parse(from_occurrences))]
    verbose: i8,

    /// Suppress all output logging. Can be used multiple times to suppress more.
    #[clap(short, long, parse(from_occurrences))]
    quiet: i8,

    /// The location of a PEM file for the identity of this server.
    #[clap(long)]
    pem: PathBuf,

    /// The address and port to bind to for the MANY Http server.
    #[clap(long, short, default_value = "127.0.0.1:8000")]
    addr: SocketAddr,

    /// Uses an ABCI application module.
    #[clap(long)]
    abci: bool,

    /// Path of a state file (that will be used for the initial setup).
    #[clap(long)]
    state: Option<PathBuf>,

    /// Path to a persistent store database (rocksdb).
    #[clap(long)]
    persistent: PathBuf,

    /// Delete the persistent storage to start from a clean state.
    /// If this is not specified the initial state will not be used.
    #[clap(long, short)]
    clean: bool,
}

fn main() {
    let Opts {
        verbose,
        quiet,
        pem,
        addr,
        abci,
        mut state,
        persistent,
        clean,
    } = Opts::parse();

    let verbose_level = 2 + verbose - quiet;
    let log_level = match verbose_level {
        x if x > 3 => LevelFilter::TRACE,
        3 => LevelFilter::DEBUG,
        2 => LevelFilter::INFO,
        1 => LevelFilter::WARN,
        0 => LevelFilter::ERROR,
        x if x < 0 => LevelFilter::OFF,
        _ => unreachable!(),
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    debug!("{:?}", Opts::parse());

    if clean {
        // Delete the persistent storage.
        // Ignore NotFound errors.
        match std::fs::remove_dir_all(persistent.as_path()) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                panic!("Error: {}", e)
            }
        }
    } else if persistent.exists() {
        // Initial state is ignored.
        state = None;
    }

    let pem = std::fs::read_to_string(&pem).expect("Could not read PEM file.");
    let key = CoseKeyIdentity::from_pem(&pem).expect("Could not generate identity from PEM file.");

    let state: Option<InitialStateJson> = state.map(|state| {
        let content = std::fs::read_to_string(&state).unwrap();
        serde_json::from_str(&content).unwrap()
    });

    let module_impl = LedgerModuleImpl::new(state, persistent, abci).unwrap();
    let module_impl = Arc::new(Mutex::new(module_impl));
    let many = ManyServer::simple(
        "many-ledger",
        key,
        Some(std::env!("CARGO_PKG_VERSION").to_string()),
    );

    {
        let mut s = many.lock().unwrap();
        s.add_module(ledger::LedgerModule::new(module_impl.clone()));
        s.add_module(ledger_transactions::LedgerTransactionsModule::new(
            module_impl.clone(),
        ));
        if abci {
            s.add_module(abci_backend::AbciModule::new(module_impl));
        }
    }

    HttpServer::new(many).bind(addr).unwrap();
}
