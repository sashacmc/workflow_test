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
use zenoh::config::Config;
use zenoh_plugin_trait::Plugin;

use crate::zenoh_args::CommonArgs;

//
// All Bridge arguments
//
#[derive(clap::Parser, Clone, Debug)]
#[command(version=zenoh_plugin_template::TemplatePlugin::PLUGIN_VERSION,
    long_version=zenoh_plugin_template::TemplatePlugin::PLUGIN_LONG_VERSION,
    about="Zenoh bridge for Template",
)]
pub struct BridgeArgs {
    #[command(flatten)]
    pub session_args: CommonArgs,

    /// Configures HTTP interface for the REST API (disabled by default, setting this option enables it). Accepted values:
    ///  - a port number
    ///  - a string with format `<local_ip>:<port_number>` (to bind the HTTP server to a specific interface).
    #[arg(short, long, value_name = "PORT | IP:PORT", verbatim_doc_comment)]
    pub rest_http_port: Option<String>,

    pub ros_args: (),
}

impl From<BridgeArgs> for Config {
    fn from(value: BridgeArgs) -> Self {
        (&value).into()
    }
}

impl From<&BridgeArgs> for Config {
    fn from(args: &BridgeArgs) -> Self {
        let mut config: Config = (&args.session_args).into();

        // if "template" plugin conf is not present, add it (empty to use default config)
        if config.plugin("template").is_none() {
            config.insert_json5("plugins/template", "{}").unwrap();
        }
        // Insert the rest plugin
        insert_json5_option(&mut config, "plugins/rest/http_port", &args.rest_http_port);

        config
    }
}

#[allow(dead_code)]
pub(crate) fn insert_json5<T>(config: &mut Config, key: &str, value: &T)
where
    T: Sized + serde::Serialize,
{
    config
        .insert_json5(key, &serde_json::to_string(value).unwrap())
        .unwrap();
}

pub(crate) fn insert_json5_option<T>(config: &mut Config, key: &str, value: &Option<T>)
where
    T: Sized + serde::Serialize,
{
    if let Some(v) = value {
        config
            .insert_json5(key, &serde_json::to_string(v).unwrap())
            .unwrap();
    }
}

#[allow(dead_code)]
pub(crate) fn insert_json5_list<T>(config: &mut Config, key: &str, values: &Vec<T>)
where
    T: Sized + serde::Serialize,
{
    if !values.is_empty() {
        config
            .insert_json5(key, &serde_json::to_string(values).unwrap())
            .unwrap();
    }
}
