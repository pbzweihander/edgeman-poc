use std::path::PathBuf;

use anyhow::Context;
use edgeman_common::pod::Pod;
use hyper::Body;
use hyperlocal::UnixConnector;
use once_cell::sync::Lazy;

static SOCKET_PATH: Lazy<String> = Lazy::new(socket_path);
static UNIX_CLIENT: Lazy<hyper::Client<UnixConnector>> = Lazy::new(|| {
    hyper::Client::builder()
        .pool_max_idle_per_host(0)
        .build(UnixConnector)
});

fn socket_path() -> String {
    let uid = nix::unistd::Uid::effective();
    let podman_user_socket = format!("/run/user/{uid}/podman/podman.sock");
    let podman_root_socket = "/run/podman/podman.sock".to_string();

    if PathBuf::from(&podman_user_socket).exists() {
        podman_user_socket
    } else {
        podman_root_socket
    }
}

async fn request(req: hyper::Request<Body>) -> anyhow::Result<hyper::Response<Body>> {
    UNIX_CLIENT
        .request(req)
        .await
        .context("failed to request to podman unix socket")
}

pub async fn play_kube(pod: Pod) -> anyhow::Result<()> {
    let req = hyper::Request::builder()
        .method("POST")
        .uri(hyperlocal::Uri::new(
            SOCKET_PATH.clone(),
            "/v4.7.0/libpod/play/kube?start=false",
        ))
        .body(Body::from(
            serde_yaml::to_string(&pod).context("failed to serialize body")?,
        ))
        .context("failed to build request")?;
    let resp = request(req).await?;
    let resp_body = hyper::body::to_bytes(resp.into_body())
        .await
        .context("failed to read response body")?;
    let resp_body_str = String::from_utf8_lossy(&resp_body);
    tracing::info!(resp = %resp_body_str, "play_kube podman response");
    Ok(())
}
