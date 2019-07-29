mod agent;
mod functions;
mod container;
mod action;

use container::Container;

fn main() {
    let mut container = Container::create(&functions::rastrigin, 4, 3, (-5.12, 5.12));
    println!("{}", container);

    //Stop condition: stop after turn_limit turns
    let turn_limit = 5;

    for turn_number in 1..=turn_limit {
        println!{"====================================== TURN {} ======================================", turn_number}
        println!{"==> Action queue at start of the turn: "}
        container.print_action_queue();

        println!{"==> Removing dead agents"}
        container.remove_dead_agents();

        println!{"==> Removing None actions"}
        container.remove_none_actions();

        println!{"==> Temporary solution: just remove those agents that want to migrate"}
        container.remove_migrants();

        println!{"==> Determining agent actions for this turn"}
        container.create_action_queue();
        println!{"Action queue in turn {} BEFORE resolution:", turn_number}
        container.print_action_queue();

        println!{"==> Resolving actions for this turn"}
        container.resolve_meetings();
        container.resolve_procreation();

        println!{"Action queue in turn {} AFTER resolution :", turn_number}
        container.print_action_queue();

        println!{"==> Executing actions for turn {}:", turn_number}
        container.execute_actions();

        println!{"==> Turn is now over. Fitness and energy of the agents at the end of turn {}:", turn_number}
        container.print_agent_stats();

        println!{"==> Action queue at the end of {}:", turn_number}
        container.print_action_queue();

        println!{"==================================== END TURN {} ====================================\n\n", turn_number}
    }
}
