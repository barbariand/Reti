use tracing_subscriber_wasm::MakeConsoleWriter;

pub fn init_logger() {
    tracing_subscriber::fmt()
        .with_writer(
            MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .without_time()
        .init()
}
