mod agent;
mod functions;
mod container;
mod action;

use container::Container;
use std::collections::HashMap;
use uuid::Uuid;
use crate::agent::Agent;

fn main() {
    let mut container = Container::create(&functions::rastrigin, 4, 3, (-5.12, 5.12));
    //println!("{}", container);

//    //Stop condition: stop after turn_limit turns
    let turn_limit = 100;
//    for turn_number in 1..=turn_limit {
//        println!{"====================================== TURN {} ======================================", turn_number}
//        println!{"==> Action queue at start of the turn: "}
//        container.print_action_queue();
//
//        println!{"==> Removing dead agents"}
//        container.remove_dead_agents();
//
//        println!{"==> Removing None actions"}
//        container.remove_none_actions();
//
//        println!{"==> Temporary solution: just remove those agents that want to migrate"}
//        container.remove_migrants();
//
//        println!{"==> Determining agent actions for this turn"}
//        container.create_action_queue();
//        println!{"Action queue in turn {} BEFORE resolution:", turn_number}
//        container.print_action_queue();
//
//        println!{"==> Resolving actions for this turn"}
//        container.resolve_meetings();
//        container.resolve_procreation();
//
//        println!{"Action queue in turn {} AFTER resolution :", turn_number}
//        container.print_action_queue();
//
//        println!{"==> Executing actions for turn {}:", turn_number}
//        container.execute_actions();
//
//        println!{"==> Turn is now over. Fitness and energy of the agents at the end of turn {}:", turn_number}
//        container.print_agent_stats();
//
//        println!{"==> Action queue at the end of {}:", turn_number}
//        container.print_action_queue();
//
//        println!{"==================================== END TURN {} ====================================\n\n", turn_number}
//    }


    //
//    let new_uuid = Uuid::new_v4();
//    let mut hm = HashMap::new();
//    hm.insert(new_uuid, Agent::create(vec![1.0, 2.0, 3.0], &functions::rastrigin));
//    let agent = hm.get_mut(&new_uuid).unwrap();
//
//    agent.energy = 90;
//
//    println!("{:?}", hm);

    let mut grid = vec![vec![Uuid::new_v4(); 3]; 4];
    grid[2][1] = Uuid::new_v4();
    for (i, row) in grid.iter().enumerate() {
        for (y, col) in row.iter().enumerate() {
            print!("{} \t", col);
        }
        println!();
    }

//    for element in grid.iter_mut().flat_map(|r| r.iter_mut()) {
//        println!("{}", element);
//    }



}
