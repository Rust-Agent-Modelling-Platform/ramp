use chrono::{Datelike, Local, Timelike};
use std::fs::File;
use std::{fs, string};
use uuid::Uuid;

use std::cell::RefCell;
use std::io::prelude::*;

use rust_in_peace::agent::Agent;
use crate::myisland::MyIsland;

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

pub fn get_best_fitness(container: &MyIsland) -> Option<f64> {
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

pub fn get_most_fit_agent(container: &MyIsland) -> &RefCell<Agent> {
    let mut top_guy = container.id_agent_map.values().take(1).last().unwrap();
    for agent in container.id_agent_map.values() {
        if agent.borrow().fitness > top_guy.borrow().fitness {
            top_guy = agent;
        }
    }
    &top_guy
}

// =================================== Stat files =========================================================
pub fn generate_stat_files(container: &MyIsland, time: u64, dir: &str) {
    //In case of decision to create a Stat struct - could be useful
    let stat_types = vec![
        "time.csv",
        "fitness.csv",
        "best_agent_overall.csv",
        "meetings.csv",
        "procreations.csv",
        "all_received_migrations.csv",
        "all_sent_migrations.csv",
        "local_sent_migrations.csv",
        "global_sent_migrations.csv",
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
    write_to_csv(
        &container.stats.meetings_in_turn,
        format!("{}/{}", dir, stat_types[3]),
    );
    write_to_csv(
        &container.stats.procreations_in_turn,
        format!("{}/{}", dir, stat_types[4]),
    );
    write_to_csv(
        &container.stats.all_received_migrations_in_turn,
        format!("{}/{}", dir, stat_types[5]),
    );
    write_to_csv(
        &container.stats.all_sent_migrations_in_turn,
        format!("{}/{}", dir, stat_types[6]),
    );
    write_to_csv(
        &container.stats.local_sent_migrations_in_turn,
        format!("{}/{}", dir, stat_types[7]),
    );
    write_to_csv(
        &container.stats.global_sent_migrations_in_turn,
        format!("{}/{}", dir, stat_types[8]),
    );
    write_to_csv(
        &container.stats.deads_in_turn,
        format!("{}/{}", dir, stat_types[9]),
    );
}

fn write_time_csv(seconds: u64, dir: String) {
    let mut file = File::create(dir).unwrap();
    writeln!(file, "{} seconds", seconds).unwrap();
}

fn write_fitness_csv(ids: &[f64], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = ids.iter().map(string::ToString::to_string).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}

fn write_best_agent_csv(agent: &Agent, dir: String) {
    let mut file = File::create(dir).unwrap();
    writeln!(file, "{}", agent).unwrap();
}

fn write_to_csv(num: &[u32], dir: String) {
    let mut file = File::create(dir).unwrap();
    let strings: Vec<String> = num.iter().map(string::ToString::to_string).collect();
    writeln!(file, "{}", strings.join(",\n")).unwrap();
}
