mod matching;
mod turbo;
mod hopcroft_karp;
mod map;
mod runner;
mod generate_gif;
mod agent_strategies;
mod target_strategies;
mod flow;
mod bench;

use crate::map::*;
use crate::runner::*;
use crate::agent_strategies::*;
use crate::target_strategies::*;
use crate::matching::*;
use crate::turbo::*;
use crate::flow::*;
use crate::bench::*;
use rand::prelude::*;

fn main() {

    let maps = vec![
        "arena.map",
        "tunnel.map",
        // "arena2.map", // too big for n4 distance oracle
    ];

    let strats = vec![
        AgentStrategies::MakeSpanHopcroft,
        AgentStrategies::NoCollisionFree,
        AgentStrategies::CollisionFree,
    ];

    let nruns = 1000;

    for map_name in maps {
        for strat in &strats {
            let map = Map::new(&("resources/maps/".to_owned() + map_name));
            let d_time = 10;

            let agent_template = AgentStrategyTemplate {
                strategy: AgentStrategies::MakeSpanHopcroft,
                permutation: None,
                matcher: None,
                flow: None,
            };

            let target_template = TargetStrategyTemplate {
                strategy: TargetStrategies::TargetFollowPath,
            };

            let res = bench(map, nruns, d_time, &mut rand::thread_rng(), 3, 3, Vec::new(), Vec::new(),
                            agent_template, target_template, false);

            match res {
                Ok(br) => {
                    println!("Benchmark finished! \nnruns: {}, map: {}, strat: {:?}", nruns, map_name, strat);
                    println!("avg length: {:.4}", br.avg_length);
                    println!("avg time: {:.4}ms", br.avg_time);
                },
                Err(s) => println!("Benchmark error: {}", s),
            }
        }

    }
}

/*
fn main() {
    let map = Map::new("resources/maps/arena.map");

    //let d_time = std::i32::MAX;
    let d_time = 2;
    /*
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
    */
    let mut agents = agents_random(&map, 3);
    let mut targets = targets_random(&map, 3, d_time);

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
*/

/*
fn main() {
    let map = Map::new("resources/maps/tunnel.map");

    let d_time = std::i32::MAX;
    let agents = agents_from(&Vec::from([
        Point{x: 1, y: 1},
    ]));
    let mut targets = targets_from(&Vec::from([
        Point{x: 27, y: 1},
    ]), d_time);

    let mut follow_path = TargetFollowPath::new(targets.len(), &map,
        targets.iter().map(|x| x.position).collect(), &mut targets, true, 20);

    let mut path = vec![Direction::West; 7];
    path.push(Direction::North);
    path.extend(vec![Direction::East; 7]);
    // follow_path.set_path(0, &path, &map, &mut targets[0], false);

    // let mut flow = FordFulkerson::new();
    let mut agent_strat = MakeSpanHopcroft {};
    // agent_strat.prep(&map, &agents, &targets, &mut flow);

    let mut runner = Runner{map: map.clone(), agents, targets, d_time};
    //let took = runner.run(MakeSpanHopcroft, follow_path, false, true, "generated/run.gif");
    let took = runner.run(Box::new(agent_strat), Box::new(follow_path), false, true, true, false, "generated/run.gif");
    println!("took: {}", took);
}
*/
