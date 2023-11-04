use edgeman_common::health::Health;

pub async fn push_health(health: Health) -> anyhow::Result<()> {
    // TODO: push health to edgeman server
    tracing::info!(?health, "STUB: pushing health");
    Ok(())
}
