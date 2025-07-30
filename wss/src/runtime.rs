use once_cell::sync::Lazy;
use tokio::runtime::{Builder, Runtime};




pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_current_thread().thread_name("tokio").enable_all().build().unwrap()
});