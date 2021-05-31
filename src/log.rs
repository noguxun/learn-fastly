mod my_log_module {
    pub fn do_a_thing() {
        log::warn!("This won't be written, because this module's max level is Error...");
        log::error!("But this will be written");
    }
}
#[allow(dead_code)]
pub enum LogSetting {
    EchoStdOutput,
    TwoEndpoints,
    FilterModule,
    Simple,
}

pub fn log_setting(log_setup: LogSetting) {
    match log_setup {
        LogSetting::EchoStdOutput => {
            log_fastly::Logger::builder()
                .max_level(log::LevelFilter::Warn)
                .default_endpoint("my_log0")
                .echo_stdout(true)
                .echo_stderr(true)
                .init();

            // this log will also be echoed to stdout and stderr
            log::warn!("hello world!");
        }

        LogSetting::TwoEndpoints => {
            log_fastly::Logger::builder()
                .max_level(log::LevelFilter::Warn)
                .default_endpoint("my_log_default")
                .endpoint("my_log2")
                .endpoint_level("my_log1", log::LevelFilter::Debug)
                .init();

            log::warn!(target: "my_log1", "log with target");
            log::warn!(target: "my_log2", "log with target");

            // without target, log goes to default log point
            log::warn!("log to default");
        }

        LogSetting::FilterModule => {
            log_fastly::Logger::builder()
                .max_level(log::LevelFilter::Info)
                .default_endpoint("my_log_default")
                .filter_module("my_.*_module", log::LevelFilter::Error)
                .init();

            log::info!("this will not be printed, not matching my_.*_module");
            my_log_module::do_a_thing();
        }

        LogSetting::Simple => {
            log_fastly::init_simple("my_log_default", log::LevelFilter::Info);
        }
    }
    // expecting an Err here for second init
    assert!(!log_fastly::Logger::builder().try_init().is_ok());

    fastly::log::set_panic_endpoint("my_log_default").unwrap();
}
