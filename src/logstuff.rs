pub fn setup_logger() {
    tracing_log::LogTracer::builder()
        .ignore_crate("foo") // suppose the `foo` crate is using `tracing`'s log feature
        .with_max_level(log::LevelFilter::Debug)
        .init();
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
