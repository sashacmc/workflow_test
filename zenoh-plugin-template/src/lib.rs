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
#![allow(deprecated)]

use std::{
    env,
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use tokio::task::JoinHandle;
use zenoh::{
    internal::{
        plugins::{RunningPlugin, RunningPluginTrait, ZenohPlugin},
        runtime::Runtime,
        zerror,
    },
    Result as ZResult,
};
use zenoh_plugin_trait::{plugin_long_version, plugin_version, Plugin, PluginControl};

pub mod config;
use config::Config;

lazy_static::lazy_static! {
    static ref WORK_THREAD_NUM: AtomicUsize = AtomicUsize::new(config::DEFAULT_WORK_THREAD_NUM);
    static ref MAX_BLOCK_THREAD_NUM: AtomicUsize = AtomicUsize::new(config::DEFAULT_MAX_BLOCK_THREAD_NUM);
    // The global runtime is used in the dynamic plugins, which we can't get the current runtime
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
               .worker_threads(WORK_THREAD_NUM.load(Ordering::SeqCst))
               .max_blocking_threads(MAX_BLOCK_THREAD_NUM.load(Ordering::SeqCst))
               .enable_all()
               .build()
               .expect("Unable to create runtime");
}
#[inline(always)]
pub(crate) fn spawn_runtime<F>(task: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // Check whether able to get the current runtime
    match tokio::runtime::Handle::try_current() {
        Ok(rt) => {
            // Able to get the current runtime (standalone binary), use the current runtime
            rt.spawn(task)
        }
        Err(_) => {
            // Unable to get the current runtime (dynamic plugins), reuse the global runtime
            TOKIO_RUNTIME.spawn(task)
        }
    }
}

#[cfg(feature = "dynamic_plugin")]
zenoh_plugin_trait::declare_plugin!(TemplatePlugin);

#[allow(clippy::upper_case_acronyms)]
pub struct TemplatePlugin;

impl ZenohPlugin for TemplatePlugin {}
impl Plugin for TemplatePlugin {
    type StartArgs = Runtime;
    type Instance = RunningPlugin;

    const PLUGIN_VERSION: &'static str = plugin_version!();
    const PLUGIN_LONG_VERSION: &'static str = plugin_long_version!();
    const DEFAULT_NAME: &'static str = "template";

    fn start(name: &str, runtime: &Self::StartArgs) -> ZResult<RunningPlugin> {
        // Try to initiate login.
        // Required in case of dynamic lib, otherwise no logs.
        // But cannot be done twice in case of static link.
        zenoh::try_init_log_from_env();

        let runtime_conf = runtime.config().lock();
        let plugin_conf = runtime_conf
            .plugin(name)
            .ok_or_else(|| zerror!("Plugin `{}`: missing config", name))?;
        let config: Config = serde_json::from_value(plugin_conf.clone())
            .map_err(|e| zerror!("Plugin `{}` configuration error: {}", name, e))?;
        WORK_THREAD_NUM.store(config.work_thread_num, Ordering::SeqCst);
        MAX_BLOCK_THREAD_NUM.store(config.max_block_thread_num, Ordering::SeqCst);

        spawn_runtime(run(runtime.clone(), config));

        Ok(Box::new(TemplatePlugin))
    }
}
impl PluginControl for TemplatePlugin {}
impl RunningPluginTrait for TemplatePlugin {}

async fn run(runtime: Runtime, config: Config) {
    // Try to initiate login.
    // Required in case of dynamic lib, otherwise no logs.
    // But cannot be done twice in case of static link.
    zenoh::try_init_log_from_env();
    tracing::debug!("Template plugin {}", TemplatePlugin::PLUGIN_VERSION);
    tracing::info!("Template plugin {config:?}");

    // open zenoh-net Session
    let _zsession = match zenoh::session::init(runtime).await {
        Ok(session) => Arc::new(session),
        Err(e) => {
            tracing::error!(
                "Unable to init zenoh session for the Template plugin : {:?}",
                e
            );
            return;
        }
    };

    // TODO: Do what plugin should do here
    println!("TODO: This is the template for plugin. Please add your own implementation here.");
}
