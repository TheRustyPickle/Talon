use tokio::runtime::{self, Runtime};

/// Convenient tokio runtime getter
pub fn get_runtime() -> Runtime {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}
