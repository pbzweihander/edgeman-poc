use anyhow::Context;
use edgeman_common::{health::Health, pod::Pod};
use serde::Deserialize;
use stopper::Stopper;
use tokio::sync::mpsc;
use url::Url;

use crate::{config::CONFIG, health::push_health, reconcile::reconcile, HTTP_CLIENT};

async fn fetch_pods(url: &Url) -> anyhow::Result<Vec<Pod>> {
    let v = match url.scheme() {
        "file" => tokio::fs::read(url.path())
            .await
            .context("failed to read fetch file path")?,
        "http" | "https" => HTTP_CLIENT
            .get(url.clone())
            .send()
            .await
            .context("failed to request to fetch URL")?
            .bytes()
            .await
            .context("failed to read body from fetch request")?
            .to_vec(),
        _ => {
            return Err(anyhow::anyhow!(
                "unsupported fetch scheme `{}`",
                url.scheme()
            ));
        }
    };

    let de = serde_yaml::Deserializer::from_slice(&v);
    Ok(de
        .filter_map(|document| match Pod::deserialize(document) {
            Ok(pod) => Some(pod),
            Err(error) => {
                tracing::warn!(?error, "failed to deserialize document to Pod. ignoring");
                None
            }
        })
        .collect())
}

async fn fetch_and_reconcile() -> anyhow::Result<()> {
    let specs = fetch_pods(&CONFIG.fetch_url)
        .await
        .context("failed to fetch pod create specs")?;

    let mut health = Health::new(CONFIG.id.clone());

    for spec in specs {
        if let Err(error) = spec.validate() {
            tracing::warn!(?error, "failed to validate pod create spec");
            continue;
        }

        let name = spec.name().to_string();
        let timestamp = spec.timestamp();

        tracing::info!(%name, %timestamp, "start reconciling");

        match reconcile(spec).await {
            Ok(()) => {
                health.pods.insert(name, timestamp);
            }
            Err(error) => {
                tracing::error!(?error, "failed to reconcile pod create spec `{}`", name);
            }
        }
    }

    push_health(health).await.context("failed to push health")
}

pub async fn fetch_loop(mut fetch_signal_rx: mpsc::Receiver<()>, stopper: Stopper) {
    while let Some(Some(_)) = stopper.stop_future(fetch_signal_rx.recv()).await {
        tracing::info!("start fetching and reconciling");
        if let Err(error) = fetch_and_reconcile().await {
            tracing::error!(?error, "failed to fetch and reconcile")
        }
    }
}
