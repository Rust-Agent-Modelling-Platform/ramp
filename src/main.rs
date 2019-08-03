mod agent;
mod functions;
mod container;
mod action;

use container::Container;
use std::collections::HashMap;
use uuid::Uuid;
use std::time::{Duration, Instant};

use crate::agent::Agent;

const TURN_LIMIT: i32 = 1000;

fn main() {
    let now = Instant::now();

    let mut container = Container::create(&functions::rastrigin, 500, 4, (-5.12, 5.12), 1000);
    for turn_number in 1..=TURN_LIMIT {
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
        container.print_agent_stats();

        println!{"==> Action queue at the end of {}:", turn_number}
        container.print_action_queue();

        println!("Total number of agents at the end of turn {}:", turn_number);
        container.print_agent_count();

        println!{"==> Removing dead agents"}
        container.remove_dead_agents();

        container.clear_action_queues();

        println!{"==================================== END TURN {} ====================================\n\n", turn_number}

    }
    println!("Time elapsed: {} seconds", now.elapsed().as_secs());

    println!("At end of simulation the best agent is:");
    container.print_most_fit_agent();

//  ======================== 2d vectors and hashmaps ==============================================


//    let new_uuid = Uuid::new_v4();
//    let mut hm = HashMap::new();
//    hm.insert(new_uuid, Agent::create(vec![1.0, 2.0, 3.0], &functions::rastrigin));
//    let agent = hm.get_mut(&new_uuid).unwrap();
//
//    agent.energy = 90;
//
//    println!("{:?}", hm);


//    let mut grid = vec![vec![Uuid::new_v4(); 3]; 4];
//    grid[2][1] = Uuid::new_v4();
//    for (i, row) in grid.iter().enumerate() {
//        for (y, col) in row.iter().enumerate() {
//            print!("{} \t", col);
//        }
//        println!();
//    }
//
////    for element in grid.iter_mut().flat_map(|r| r.iter_mut()) {
////        println!("{}", element);
////    }



}
