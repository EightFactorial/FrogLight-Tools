use tracing_subscriber::{fmt::SubscriberBuilder, EnvFilter};

#[allow(dead_code)]
const DEBUG_LOG_LEVEL: &str = "trace";
#[allow(dead_code)]
const RELEASE_LOG_LEVEL: &str = "debug";

/// Setup logging.
pub(super) fn setup() {
    let builder = SubscriberBuilder::default().without_time().compact();

    let filter: EnvFilter;
    #[cfg(debug_assertions)]
    {
        filter = EnvFilter::from_default_env()
            .add_directive("reqwest=warn".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap())
            .add_directive(format!("froglight_extractor={DEBUG_LOG_LEVEL}").parse().unwrap())
            .add_directive(format!("froglight_generator={DEBUG_LOG_LEVEL}").parse().unwrap());
    }
    #[cfg(not(debug_assertions))]
    {
        filter = EnvFilter::from_default_env()
            .add_directive("reqwest=warn".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap())
            .add_directive(format!("froglight_extractor={RELEASE_LOG_LEVEL}").parse().unwrap())
            .add_directive(format!("froglight_generator={RELEASE_LOG_LEVEL}").parse().unwrap());
    }

    builder.with_env_filter(filter).init();
}
