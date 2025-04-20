use leptos::mount::mount_to_body;
use tracing_subscriber::{fmt, util::SubscriberInitExt};
use tracing_subscriber_wasm::MakeConsoleWriter;

fn main() {
    use laim::App;

    console_error_panic_hook::set_once();
    fmt()
        .with_max_level(tracing::Level::TRACE)
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_writer(MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG))
        .with_ansi(false)
        .pretty()
        .finish()
        .init();

    mount_to_body(App)
}
