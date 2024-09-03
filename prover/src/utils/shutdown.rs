use crate::threadpool::ThreadPool;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::info; // Import the logging macro

pub async fn shutdown_signal(thread_pool: Arc<Mutex<ThreadPool>>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Shutting down the server");
        },
        _ = terminate => {
            info!("Shutting down the server due to termination signal");
        },
    }

    // Trigger thread pool shutdown
    info!("Shutting down the thread pool...");
    let mut thread_pool = thread_pool.lock().await;
    if let Err(e) = thread_pool.shutdown().await {
        eprintln!("Error during thread pool shutdown: {:?}", e);
    } else {
        info!("Thread pool shutdown completed successfully.");
    }
}
