use once_cell::sync::Lazy;
use tokio::runtime::{Builder, Runtime};
pub mod matching;
use matching::*;





// custom tokio runtime to be used in syn context
pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread().thread_name("tokio").enable_all().build().unwrap()
});

