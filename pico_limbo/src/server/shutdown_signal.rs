use tokio_util::sync::CancellationToken;

pub async fn shutdown_signal(token: Option<&CancellationToken>) {
    match token {
        None => platform_signal().await,
        Some(token) => {
            token.cancelled().await;
        }
    }
}

async fn platform_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = sigint.recv() => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        use tokio::signal;
        // On Windows, tokio::signal::ctrl_c is the best we can do.
        let _ = signal::ctrl_c().await;
    }
}
