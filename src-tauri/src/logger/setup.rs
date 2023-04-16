use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::{
            policy::compound::{
                CompoundPolicy,
                roll::fixed_window::FixedWindowRoller,
                // roll::delete::DeleteRoller,
                trigger::size::SizeTrigger,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use log::LevelFilter;

use crate::utils;

pub fn setup_logger() {
    // define log levels
    let global_log_level = LevelFilter::Debug;
    let stdout_log_level = LevelFilter::Debug;
    let logfile_log_level;
    if cfg!(debug_assertions) {
        // on dev mode, set file log to debug as well
        logfile_log_level = LevelFilter::Debug;
    } else {
        logfile_log_level = LevelFilter::Info;
    }

    // define log pattern
    let log_pattern = "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}";

    // define single log file size to 10MB
    let trigger_size = byte_unit::n_mb_bytes!(10) as u64;
    let trigger = SizeTrigger::new(trigger_size);

    // config rolling params
    // rolling file like: 1.log 2.log ...
    let roller_pattern = "ai_voice_factory-{}.log";
    // rolling total files
    let roller_count = 10;
    // rolling index initial number
    let roller_base = 1;
    let roller = FixedWindowRoller::builder()
        .base(roller_base)
        .build(roller_pattern, roller_count).unwrap();

    // compound trigger and roller
    let log_file_compound_policy = CompoundPolicy::new(Box::new(trigger),
                                                       Box::new(roller));

    // define rolling file appender
    let log_path = utils::get_app_home_dir().join("logs").join("ai_voice_factory.log");
    let log_file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build(log_path, Box::new(log_file_compound_policy))
        .unwrap();

    // define console appender
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .target(Target::Stdout)
        .build();

    // config logger
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(logfile_log_level)))
                .build("log_file", Box::new(log_file)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(stdout_log_level)))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("log_file")
                .build(global_log_level))
        .unwrap();

    let _log_handler = log4rs::init_config(config).unwrap();

    log::debug!("Logger initialized");
}
