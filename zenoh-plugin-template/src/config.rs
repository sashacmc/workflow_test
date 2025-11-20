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
//

use serde::{de, Deserialize, Deserializer};

pub const DEFAULT_WORK_THREAD_NUM: usize = 2;
pub const DEFAULT_MAX_BLOCK_THREAD_NUM: usize = 50;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default = "default_work_thread_num")]
    pub work_thread_num: usize,
    #[serde(default = "default_max_block_thread_num")]
    pub max_block_thread_num: usize,
    __required__: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_path")]
    __path__: Option<Vec<String>>,
}

fn default_work_thread_num() -> usize {
    DEFAULT_WORK_THREAD_NUM
}

fn default_max_block_thread_num() -> usize {
    DEFAULT_MAX_BLOCK_THREAD_NUM
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(OptPathVisitor)
}

struct OptPathVisitor;

impl<'de> serde::de::Visitor<'de> for OptPathVisitor {
    type Value = Option<Vec<String>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "none or a string or an array of strings")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PathVisitor).map(Some)
    }
}

struct PathVisitor;

impl<'de> serde::de::Visitor<'de> for PathVisitor {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string or an array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(vec![v.into()])
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut v = if let Some(l) = seq.size_hint() {
            Vec::with_capacity(l)
        } else {
            Vec::new()
        };
        while let Some(s) = seq.next_element()? {
            v.push(s);
        }
        Ok(v)
    }
}
