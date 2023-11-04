use anyhow::Context;
use chrono::Utc;
use edgeman_common::pod::Pod;

use crate::podman::play_kube;

pub async fn reconcile(pod: Pod) -> anyhow::Result<()> {
    let now = Utc::now();
    let name = pod.name().to_string();
    let timestamp = pod.timestamp();
    if now < timestamp {
        tracing::warn!(%name, %timestamp, "timestamp is greater than now. ignoring");
        Ok(())
    } else {
        play_kube(pod).await.context("failed to play kube")?;
        tracing::info!(%name, %timestamp, "reconciled");
        Ok(())
    }
}
