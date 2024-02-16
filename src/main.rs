mod matching;
mod turbo;
mod hopcroft_karp;
mod map;
mod runner;
mod generate_gif;

use crate::map::*;
use crate::runner::*;

fn main() {
    let map = Map::new("resources/maps/example.map");
    let agents = Vec::from([Agent{position: Point{x: 1, y: 1}}]);
    let targets = Vec::from([Target{position: Point{x: 3, y: 3}, timer: 2}]);
    let mut runner = Runner{map, agents, targets, d_time: 2};
    let took = runner.run(MakeSpanHopcroft, RandomTarget, false, true, "generated/run.gif");
    println!("took: {}", took);
}
