use chrono::{Datelike, Local, Timelike};
use flexi_logger::Logger;
use std::fs::File;
use std::{env, fs};
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

pub fn create_simulation_dir(root_dir_path: &str) -> String {
    let now = Local::now();
    let hour = now.hour();
    let (_, year) = now.year_ce();

    let simulation_dir_name = format!(
        "{}-{:0>2}-{:0>2}_{:0>2}{:0>2}{:0>2}",
        year.to_string(),
        now.month().to_string(),
        now.day().to_string(),
        hour.to_string(),
        now.minute().to_string(),
        now.second().to_string()
    );
    let simulation_dir_path = format!("{}/{}", &root_dir_path, &simulation_dir_name);
    match fs::create_dir_all(simulation_dir_path.clone()) {
        Err(e) => eprintln!("{}", e),
        Ok(_) => log::info!("Created directory for simulation: {}", &simulation_dir_name),
    }
    simulation_dir_path
}

pub fn copy_simulation_settings(dest_dir_path: &str, file_name: String) {
    let dest_file_path = format!("{}/{}", dest_dir_path, file_name);
    let _file = File::create(&dest_file_path).unwrap();
    fs::copy(file_name.to_string(), dest_file_path).unwrap();
}

pub fn create_island_stats_dir(simulation_dir_path: &str, island_id: &Uuid) -> String {
    let path = format!(
        "{}/Island-{}",
        &simulation_dir_path,
        &island_id.to_string()[..5]
    );
    match fs::create_dir(&path) {
        Err(e) => eprintln!("{}", e),
        Ok(_) => log::info!(
            "Created directory for Island-{}",
            &island_id.to_string()[..5]
        ),
    }
    path
}
