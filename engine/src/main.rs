use engine::TOKIO_RUNTIME;
use scylla::client::session_builder::SessionBuilder;







fn main() {
    let session = TOKIO_RUNTIME.block_on(
        SessionBuilder::new().known_node("127.0.0.1:9042").build()
    ).unwrap();
    
}
