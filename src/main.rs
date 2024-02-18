mod matching;
mod turbo;
mod hopcroft_karp;
mod map;
mod runner;
mod generate_gif;
mod agent_strategies;
mod target_strategies;

use crate::map::*;
use crate::runner::*;
use crate::agent_strategies::*;
use crate::target_strategies::*;

fn main() {
    let map = Map::new("resources/maps/arena.map");

    let d_time = 2;
    /*
    let agents = agents_from(&Vec::from([
        Point{x: 3, y: 2},
        Point{x: 3, y: 47},
    ]));
    let targets = targets_from(&Vec::from([
        Point{x: 10, y: 10},
        Point{x: 47, y: 47},
    ]), d_time);
    */
    let agents = agents_random(&map, 3);
    let targets = targets_random(&map, 3, d_time);

    let mut runner = Runner{map, agents, targets, d_time};
    let took = runner.run(MakeSpanHopcroft, RandomTarget, false, true, "generated/run.gif");
    println!("took: {}", took);
}
