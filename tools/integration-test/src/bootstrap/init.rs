/*!
   Functions for initializing each test at the beginning of a Rust test
   session.
*/

use eyre::Report as Error;
use std::env;
use std::fs;
use std::sync::Once;
use tracing_subscriber::{
    self as ts,
    filter::EnvFilter,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
};

use crate::types::config::TestConfig;
use crate::util::random::random_u32;

static INIT: Once = Once::new();

/**
   Initialize the test with a global logger and error handlers,
   read the environment variables and return a [`TestConfig`].
*/
pub fn init_test() -> Result<TestConfig, Error> {
    INIT.call_once(|| {
        color_eyre::install().unwrap();
        install_logger();
    });

    let chain_command_path = env::var("CHAIN_COMMAND_PATH").unwrap_or_else(|_| "gaiad".to_string());

    let base_chain_store_dir = env::var("CHAIN_STORE_DIR").unwrap_or_else(|_| "data".to_string());

    let chain_store_dir = format!("{}/test-{}", base_chain_store_dir, random_u32());

    fs::create_dir_all(&chain_store_dir)?;

    let chain_store_dir = fs::canonicalize(chain_store_dir)?;

    let hang_on_fail = env::var("HANG_ON_FAIL")
        .ok()
        .map(|val| val == "1")
        .unwrap_or(false);

    Ok(TestConfig {
        chain_command_path,
        chain_store_dir,
        hang_on_fail,
        bootstrap_with_random_ids: true,
    })
}

/**
   Install the [`tracing_subscriber`] logger handlers so that logs will
   be displayed during test.
*/
pub fn install_logger() {
    // Use log level INFO by default if RUST_LOG is not set.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let module_filter_fn = ts::filter::filter_fn(|metadata| match metadata.module_path() {
        Some(path) => path.starts_with("ibc"),
        None => false,
    });

    let module_filter = ts::fmt::layer().with_filter(module_filter_fn);

    ts::registry().with(env_filter).with(module_filter).init();
}
