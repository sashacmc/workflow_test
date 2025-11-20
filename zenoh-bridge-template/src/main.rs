//
// Copyright (c) 2025 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use bridge_args::BridgeArgs;
use clap::Parser;
use zenoh::{
    config::Config,
    internal::{plugins::PluginsManager, runtime::RuntimeBuilder},
};
use zenoh_config::ModeDependentValue;
use zenoh_plugin_trait::Plugin;

mod bridge_args;
mod zenoh_args;

fn parse_args() -> Config {
    // Create config parsing user-defined args
    let bridge_args = BridgeArgs::parse_from(std::env::args());
    let mut config: Config = bridge_args.into();

    // Always add timestamps to publications (required for PublicationCache used in case of TRANSIENT_LOCAL topics)
    config
        .timestamping
        .set_enabled(Some(ModeDependentValue::Unique(true)))
        .unwrap();

    // Enable admin space
    config.adminspace.set_enabled(true).unwrap();
    // Enable loading plugins
    config.plugins_loading.set_enabled(true).unwrap();

    config
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("z=info");

    tracing::info!(
        "zenoh-bridge-template {}",
        zenoh_plugin_template::TemplatePlugin::PLUGIN_LONG_VERSION
    );

    let config = parse_args();
    tracing::info!("Zenoh {config:?}");

    let mut plugins_mgr = PluginsManager::static_plugins_only();

    // declare REST plugin if specified in conf
    if config.plugin("rest").is_some() {
        plugins_mgr.declare_static_plugin::<zenoh_plugin_rest::RestPlugin, &str>("rest", true);
    }

    // declare Template plugin
    plugins_mgr
        .declare_static_plugin::<zenoh_plugin_template::TemplatePlugin, &str>("template", true);

    // create a zenoh Runtime.
    let mut runtime = match RuntimeBuilder::new(config)
        .plugins_manager(plugins_mgr)
        .build()
        .await
    {
        Ok(runtime) => runtime,
        Err(e) => {
            println!("{e}. Exiting...");
            std::process::exit(-1);
        }
    };
    if let Err(e) = runtime.start().await {
        println!("Failed to start Zenoh runtime: {e}. Exiting...");
        std::process::exit(-1);
    }

    futures::future::pending::<()>().await;
}
