use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn setup_logger() {
    // setup tracing and log trace events
    let subscriber = FmtSubscriber::builder()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        // .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing_log::LogTracer::builder()
        .ignore_crate("foo") // suppose the `foo` crate is using `tracing`'s log feature
        .with_max_level(log::LevelFilter::Debug)
        .init()
        .expect("Unable to setup LogTracer");
    /*
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                record.level(),
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S:%.3f %Z"),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    */
}
