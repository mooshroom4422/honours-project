mod matching;
mod turbo;
mod hopcroft_karp;
mod map;
mod runner;
mod generate_gif;
mod agent_strategies;
mod target_strategies;
mod flow;

use crate::map::*;
use crate::runner::*;
use crate::agent_strategies::*;
use crate::target_strategies::*;
use crate::matching::*;
use crate::turbo::*;
use crate::flow::*;

fn main() {
    let map = Map::new("resources/maps/arena.map");

    //let d_time = std::i32::MAX;
    let d_time = 2;
    let agents = agents_from(&Vec::from([
        Point{x: 3, y: 2},
        Point{x: 3, y: 47},
        Point{x: 10, y: 30},
        Point{x: 30, y: 10},
    ]));
    let mut targets = targets_from(&Vec::from([
        Point{x: 10, y: 10},
        Point{x: 30, y: 20},
        Point{x: 20, y: 30},
        Point{x: 47, y: 47},
    ]), d_time);

    let mut follow_path = TargetFollowPath::new(targets.len(), &map,
        targets.iter().map(|x| x.position).collect(), &mut targets, true, 100);

    let perm = vec![1, 3, 2, 0];
    // let mut agent_strat = CollisionAssigned::new();
    // agent_strat.prep(&map, &agents, &targets, &perm);

    let mut flow = FordFulkerson::new();
    let mut agent_strat = NoCollisionFree::new();
    agent_strat.prep(&map, &agents, &targets, &mut flow);

    //let mut matcher = TurboMatching::new_empty();
    //let mut agent_strat = CollisionFree::new();
    //agent_strat.prep(&map, &agents, &targets, &mut matcher);

    let mut runner = Runner{map: map.clone(), agents, targets, d_time};
    let took = runner.run(agent_strat, follow_path, false, true, "generated/run.gif");
    //let took = runner.run(MakeSpanHopcroft, follow_path, false, true, "generated/run.gif");
    println!("took: {}", took);
}
/*
fn main() {
    let map = Map::new("resources/maps/box.map");

    let d_time = std::i32::MAX;
    let agents = agents_from(&Vec::from([
        Point{x: 1, y: 1},
    ]));
    let mut targets = targets_from(&Vec::from([
        Point{x: 8, y: 8},
    ]), d_time);

    let mut follow_path = TargetFollowPath::new(targets.len(), &map,
        targets.iter().map(|x| x.position).collect(), &mut targets, false, 0);

    let mut path = vec![Direction::West; 7];
    path.push(Direction::North);
    path.extend(vec![Direction::East; 7]);
    follow_path.set_path(0, &path, &map, &mut targets[0], false);

    let mut flow = FordFulkerson::new();
    let mut agent_strat = NoCollisionFree::new();
    agent_strat.prep(&map, &agents, &targets, &mut flow);

    let mut runner = Runner{map: map.clone(), agents, targets, d_time};
    //let took = runner.run(MakeSpanHopcroft, follow_path, false, true, "generated/run.gif");
    let took = runner.run(agent_strat, follow_path, false, true, "generated/run.gif");
    println!("took: {}", took);
}
*/
