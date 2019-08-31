use chrono::{Datelike, Local, Timelike};
use std::fs;
use std::fs::File;
use uuid::Uuid;

use std::cell::RefCell;
use std::io::prelude::*;

use crate::agent::Agent;
use crate::container::Container;

// =================================== Info-generating methods =========================================================

//pub fn print_action_queues(container: &Container) {
//    println!("Dead agents:");
//    for agent in &container.dead_ids {
//        println!("{}", agent.to_string());
//        println!(
//            "Nr of entries in this queue: {}",
//            container.action_queue.len()
//        );
//    }
//    println!("Procreating agents:");
//    for agent in &container.procreating_ids {
//        println!("{}", agent.0.to_string());
//        println!(
//            "Nr of entries in this queue: {}",
//            container.action_queue.len()
//        );
//    }
//    println!("Meeting agents:");
//    for agent in &container.meeting_ids {
//        println!("{}", agent.0.to_string());
//        println!(
//            "Nr of entries in this queue: {}",
//            container.action_queue.len()
//        );
//    }
//}

//pub fn print_agent_stats(container: &Container) {
//    for (_id, agent) in &container.id_agent_map {
//        println!(
//            "Agent {}: Fitness - {}, energy - {}",
//            &agent.id.to_string()[..5],
//            agent.fitness,
//            agent.energy
//        )
//    }
//}

//pub fn print_agent_count(container: &Container) {
//    println!("{}", container.id_agent_map.len());
//}

pub fn get_best_fitness(container: &Container) -> Option<f64> {
    let mut top_guy = match container.id_agent_map.values().take(1).last() {
        Some(a) => a,
        None => return None,
    };
    for agent in container.id_agent_map.values() {
        if agent.borrow().fitness > top_guy.borrow().fitness {
            top_guy = agent;
        }
    }
    Some(top_guy.borrow().fitness)
}

// pub fn print_best_fitness(container: &Container) {
//     let mut top_guy = container
//         .id_agent_map
//         .values()
//         .take(1)
//         .last()
//         .expect("No more agents in system");
//     for agent in container.id_agent_map.values() {
//         if agent.borrow().fitness > top_guy.borrow().fitness {
//             top_guy = agent;
//         }
//     }
//     log::info!("{}", top_guy.borrow().fitness.to_string().blue());
// }

pub fn get_most_fit_agent(container: &Container) -> &RefCell<Agent> {
    let mut top_guy = container.id_agent_map.values().take(1).last().unwrap();
    for agent in container.id_agent_map.values() {
        if agent.borrow().fitness > top_guy.borrow().fitness {
            top_guy = agent;
        }
    }
    &top_guy
}

// =================================== Stat files =========================================================
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

pub fn generate_stat_files(container: &Container, time: u64, dir: &str) {
    //In case of decision to create a Stat struct - could be useful
    let stat_types = vec![
        "time.csv",
        "fitness.csv",
        "best_agent_overall.csv",
        "meetings.csv",
        "procreations.csv",
        "migrations.csv",
        "deads.csv",
    ];

    write_time_csv(time, format!("{}/{}", dir, stat_types[0]));
    write_fitness_csv(
        &container.stats.best_fitness_in_turn,
        format!("{}/{}", dir, stat_types[1]),
    );
    write_best_agent_csv(
        &get_most_fit_agent(container).borrow_mut(),
        format!("{}/{}", dir, stat_types[2]),
    );
    write_meetings_csv(
        &container.stats.meetings_in_turn,
        format!("{}/{}", dir, stat_types[3]),
    );
    write_procreations_csv(
        &container.stats.procreations_in_turn,
        format!("{}/{}", dir, stat_types[4]),
    );
    write_migrations_csv(
        &container.stats.migrants_received_in_turn,
        format!("{}/{}", dir, stat_types[5]),
    );
    write_deads_csv(
        &container.stats.deads_in_turn,
        format!("{}/{}", dir, stat_types[6]),
    );
}

fn write_time_csv(seconds: u64, dir: String) {
    let mut file = File::create(dir).unwrap();
    writeln!(file, "{} seconds", seconds).unwrap();
}

fn write_fitness_csv(ids: &[f64], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = ids.iter().map(|n| n.to_string()).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}

fn write_best_agent_csv(agent: &Agent, dir: String) {
    let mut file = File::create(dir).unwrap();
    writeln!(file, "{}", agent).unwrap();
}

fn write_meetings_csv(num: &[u32], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = num.iter().map(|n| n.to_string()).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}

fn write_procreations_csv(num: &[u32], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = num.iter().map(|n| n.to_string()).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}

fn write_migrations_csv(num: &[u32], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = num.iter().map(|n| n.to_string()).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}

fn write_deads_csv(num: &[u32], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = num.iter().map(|n| n.to_string()).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}
