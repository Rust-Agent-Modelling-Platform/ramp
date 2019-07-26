mod agent;
mod functions;
mod container;

use container::Container;

fn main() {
    let container = Container::create(&functions::rastrigin, 50, 5, (-5.12, 5.12));
    println!("{}", container);
}
