use tokio::runtime::{self, Runtime};

/// Convenient tokio runtime getter
pub fn get_runtime() -> Runtime {
    runtime::Builder::new_multi_thread()
        .worker_threads(99)
        .enable_all()
        .build()
        .unwrap()
}
