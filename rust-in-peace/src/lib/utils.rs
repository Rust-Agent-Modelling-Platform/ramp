use flexi_logger::Logger;
use std::env;
use uuid::Uuid;

pub fn init_logger(logger_lever: &str) {
    Logger::with_str(logger_lever)
        .format_for_stderr(flexi_logger::colored_default_format)
        .start()
        .unwrap();
}

pub fn parse_input_args(expected_args_num: usize) -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), expected_args_num);
    args
}

pub fn short_id(id: &Uuid) -> String {
    String::from(&id.to_string()[..5])
}
