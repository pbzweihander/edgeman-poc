use once_cell::sync::Lazy;
use stopper::Stopper;
use tokio::{
    signal::unix::{signal, Signal, SignalKind},
    sync::mpsc,
};
use tokio_cron_scheduler::{Job, JobScheduler};

mod config;
mod fetch;
mod health;
mod podman;
mod reconcile;

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent(format!(
            "{}/{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            config::CONFIG.id,
        ))
        .build()
        .expect("failed to build HTTP client")
});

async fn shutdown_signal(mut sigterm: Signal, stopper: Stopper) {
    let ctrlc = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    let sigterm = async {
        sigterm.recv().await;
    };

    tokio::select! {
        _ = ctrlc => {},
        _ = sigterm => {},
    }

    stopper.stop();
}

async fn sighup_loop(mut sighup: Signal, fetch_signal_tx: mpsc::Sender<()>, stopper: Stopper) {
    while let Some(Some(_)) = stopper.stop_future(sighup.recv()).await {
        sighup.recv().await;
        let _ = fetch_signal_tx.send(()).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (fetch_signal_tx, fetch_signal_rx) = mpsc::channel::<()>(4);
    let stopper = Stopper::new();

    let sigterm = signal(SignalKind::terminate())?;
    tokio::spawn(shutdown_signal(sigterm, stopper.clone()));

    let sighup = signal(SignalKind::hangup())?;
    tokio::spawn(sighup_loop(
        sighup,
        fetch_signal_tx.clone(),
        stopper.clone(),
    ));

    let sched = JobScheduler::new().await?;
    sched
        .add(Job::new_async(
            config::CONFIG.fetch_schedule.clone(),
            move |_, _| {
                let fetch_signal_tx = fetch_signal_tx.clone();
                Box::pin(async move {
                    let _ = fetch_signal_tx.send(()).await;
                })
            },
        )?)
        .await?;
    sched.shutdown_on_ctrl_c();
    sched.start().await?;

    tracing::info!("starting main fetch loop");
    fetch::fetch_loop(fetch_signal_rx, stopper).await;

    Ok(())
}
