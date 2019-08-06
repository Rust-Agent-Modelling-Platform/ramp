#[macro_use]
extern crate serde_derive;

mod agent;
mod functions;
mod container;
mod action;
mod settings;

use container::Container;
use std::collections::HashMap;
use uuid::Uuid;
use std::time::{Duration, Instant};
use config;
use settings::Settings;
use config::ConfigError;

use crate::agent::Agent;

fn main() -> Result<(), ConfigError> {
    let settings = Settings::new()?;

    let now = Instant::now();
    
    let mut container = Container::new(&functions::rastrigin, settings.container.agents_number, 50, (-5.12, 5.12), settings.container.max_agents_number);
    for turn_number in 1..=settings.iterations {
        println!{"====================================== TURN {} ======================================", turn_number}
        println!{"==> Action queue at start of the turn: "}
        container.print_action_queue();

        println!{"==> Temporary solution: just remove those agents that want to migrate"}
        container.remove_migrants();

        println!{"==> Determining agent actions for this turn"}
        container.create_action_queues();
        println!{"Action queue in turn {} BEFORE resolution:", turn_number}
        container.print_action_queue();

        println!{"==> Resolving actions for this turn"}
        container.resolve_procreation();
        container.resolve_meetings();

        println!{"==> Turn is now over. Fitness and energy of the agents at the end of turn {}:", turn_number}
        //container.print_agent_stats();

        println!{"==> Action queue at the end of {}:", turn_number}
        container.print_action_queue();

        println!("Total number of agents at the end of turn {}:", turn_number);
        container.print_agent_count();

        println!{"==> Removing dead agents"}
        container.remove_dead_agents();

        println!("At end of turn the best agent is:");
        container.print_most_fit_agent();

        container.clear_action_queues();

        println!{"==================================== END TURN {} ====================================\n\n", turn_number}

    }
    println!("Time elapsed: {} seconds", now.elapsed().as_secs());

    println!("At end of simulation the best agent is:");
    container.print_most_fit_agent();

    Ok(())
}
