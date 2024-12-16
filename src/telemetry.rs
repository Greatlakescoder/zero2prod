use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing::subscriber::set_global_default;
use tracing_subscriber::fmt::MakeWriter;

// Compose multiple layers into a tracings subscriber
// We are using impl Subsciber as return type to avoid having
// to spell out actual type of the returned subscriber, need to also
// say it Send + Sync for init_subscriber usage

pub fn get_subscriber<Sink>(name: String, env_filter: String, sink: Sink) -> impl Subscriber + Send + Sync where Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static, {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all logs events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    set_global_default(subscriber).expect("Failed to set subscriber");
}
