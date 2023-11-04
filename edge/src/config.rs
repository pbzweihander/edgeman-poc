use std::str::FromStr;

use anyhow::Context;
use cron::Schedule;
use once_cell::sync::Lazy;
use serde::{de::Error, Deserialize, Deserializer};
use url::Url;

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env().unwrap());

fn deserialize_fetch_url<'de, D>(d: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let url = Url::deserialize(d)?;
    if !["file", "http", "https"].contains(&url.scheme()) {
        Err(D::Error::custom(format!(
            "unsupported fetch scheme: `{}`",
            url.scheme()
        )))
    } else {
        Ok(url)
    }
}

fn deserialize_fetch_schedule<'de, D>(d: D) -> Result<Schedule, D::Error>
where
    D: Deserializer<'de>,
{
    let schedule = String::deserialize(d)?;
    Schedule::from_str(&schedule).map_err(D::Error::custom)
}

#[derive(Deserialize)]
pub struct Config {
    pub id: String,
    pub edgeman_url: Url,
    #[serde(deserialize_with = "deserialize_fetch_url")]
    pub fetch_url: Url,
    #[serde(deserialize_with = "deserialize_fetch_schedule")]
    pub fetch_schedule: Schedule,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        envy::from_env().context("failed to parse environment variables")
    }
}
