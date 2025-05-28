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
use hopcroft_karp::HopcroftKarp;
use rand::prelude::*;
use log::*;
use env_logger::fmt::{Color, Formatter};
use env_logger::Builder;
use std::io::Write;
use std::time::Instant;

fn custom_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let mut style = buf.style();
    let prefix = match record.level() {
        Level::Error => {
            style.set_color(Color::Red).set_bold(true);
            "E"
        }
        Level::Warn => {
            style.set_color(Color::Yellow).set_bold(true);
            "W"
        }
        Level::Info => {
            style.set_color(Color::Green);
            "*"
        }
        Level::Debug => {
            style.set_color(Color::Blue);
            "D"
        }
        Level::Trace => {
            style.set_color(Color::Magenta);
            "T"
        }
    };

    writeln!(buf, "[{}] {}", style.value(prefix), record.args())
}

fn main() {
    Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(custom_format)
        .init();
    let maps = vec![
//        "simple.map",
//        "arena.map",
//        "tunnel.map",
//        "den020d.map",
//        "den101d.map",
//        "den202d.map",
        "den312d.map",
//        "den998d.map",
//        "arena2.map", // too big for n^4 distance oracle
    ];

    let strats = vec![
        AgentStrategies::MakeSpanHopcroft,
        // AgentStrategies::NoCollisionFree,
        // AgentStrategies::CollisionFree,
    ];

    let nruns = 10_000;

    for map_name in maps {

        // TODO: for some reason optimal method is worse if num_agents > 1
        info!("benchmarking map: {}", map_name);

        let d_time = 15;
        let num_agents = 3;
        let num_targets = 3;
        let map = Map::new(&("resources/maps/".to_owned() + map_name));
        trace!("generating target set");
        let set = gen_set(&map, nruns, d_time, num_agents, num_targets, &mut rand::thread_rng(), Vec::new(), Vec::new());
        trace!("done!");
        if set.is_err() {
            println!("Failed to generate test set: {}", set.unwrap_err());
            return;
        }

        let (all_agents, mut all_targets) = set.unwrap();

        // println!("{} {}", all_agents.len(), all_targets.len());

        let mut strategies: Vec<Box<dyn TargetStrategy>> = Vec::new();

        trace!("generating target paths");
        let start = Instant::now();
        for targets in &mut all_targets {
            let target_strategy = TargetFollowPath::new(targets.len(), &map,
                targets.iter().map(|x| x.position).collect(), targets, true, 0);
            strategies.push(Box::new(target_strategy));
        }
        trace!("done! took: {:?}", start.elapsed());

        let mut collected: Vec<Vec<u64>> = Vec::new();
        for strat in &strats {
            info!("benchmarking strategy: {:?}", strat);

            let agent_template = AgentStrategyTemplate {
                strategy: strat.clone(),
                permutation: None,
                matcher: Some(HopcroftKarp::new()),
                flow: Some(FordFulkerson::new()),
            };

            for strat in &mut strategies {
                strat.flush();
            }

            let res = bench(map.clone(), nruns as i32, d_time, all_agents.clone(),
                            all_targets.clone(), agent_template, &mut strategies, false, true);

            match res {
                Ok(br) => {
                    println!("Benchmark finished! \nnruns: {}, map: {}, strat: {:?}", nruns, map_name, strat);
                    println!("avg length: {:.4}", br.avg_length);
                    println!("avg time: {:.4}ms", br.avg_time);
                    collected.push(br.all_results);
                },
                Err(s) => println!("Benchmark error: {}", s),
            }
        }
    
        /*
        for i in 0..nruns {
            if collected[0][i] < collected[1][i] && true {
                println!("invalid: {} {:?} {:?}", i, collected[0][i], collected[1][i]);
                println!("{:?}, {:?}", all_agents[i], all_targets[i]);

                let mut agent_strat = CollisionFree::new();
                let mut matcher = HopcroftKarp::new();
                agent_strat.prep(&map, &mut all_agents[i].clone(), &all_targets[i], &mut matcher);

                let mut runner = Runner {
                    map: map.clone(),
                    agents: all_agents[i].clone(),
                    targets: all_targets[i].clone(),
                    d_time
                };

                strategies[i].flush();
                let mut took_steps = runner.run(Box::new(agent_strat), &mut strategies[i], false, false, true, false, "generated/opt.gif", 3000) as u64;
                println!("took: {}", took_steps);

                let mut runner = Runner {
                    map: map.clone(),
                    agents: all_agents[i].clone(),
                    targets: all_targets[i].clone(),
                    d_time
                };

                strategies[i].flush();
                let hophop = MakeSpanHopcroft {};
                let mut took_steps = runner.run(Box::new(hophop), &mut strategies[i], true, false, true, true, "generated/hop.gif", 3000) as u64;
                println!("took: {}", took_steps);

                strategies[i].flush();
                let mut perm = CollisionAssigned::new();
                perm.prep(&map, &mut all_agents[i].clone(), &all_targets[i], &vec![0, 1]);
                took_steps = runner.run(Box::new(perm.clone()), &mut strategies[i], false, false, false, false, "", 3000) as u64;
                println!("01 took: {}", took_steps);

                strategies[i].flush();
                perm.prep(&map, &mut all_agents[i].clone(), &all_targets[i], &vec![1, 0]);
                took_steps = runner.run(Box::new(perm.clone()), &mut strategies[i], false, false, false, false, "", 3000) as u64;
                println!("10 took: {}", took_steps);

                return;
            }
        }
        */
    }
}

/*
fn main() {
    env_logger::init();
    let map = Map::new("resources/maps/den312d.map");


    let d_time = std::i32::MAX;
    // let d_time = 2;
    let agents = agents_from(&Vec::from([
        Point { x: 36, y: 14 },
        Point { x: 42, y: 39 },
        Point { x: 17, y: 15 },
    ]));
    let mut targets = targets_from(&Vec::from([
        Point { x: 24, y: 60 },
        Point { x: 63, y: 4 },
        Point { x: 49, y: 5 },
    ]), d_time);
    // let mut agents = agents_random(&map, 3);
    // let mut targets = targets_random(&map, 3, d_time);

    let mut follow_path: Box<dyn TargetStrategy> =
        Box::new(TargetFollowPath::new(targets.len(), &map,
        targets.iter().map(|x| x.position).collect(), &mut targets, true, 0));

    let perm = vec![1, 3, 2, 0];
    // let mut agent_strat = CollisionAssigned::new();
    // agent_strat.prep(&map, &agents, &targets, &perm);

    let mut flow = FordFulkerson::new();
    // let mut agent_strat = NoCollisionFree::new();
    let mut agent_strat = MakeSpanHopcroft {} ;
    // agent_strat.prep(&map, &agents, &targets, &mut flow);

    //let mut matcher = TurboMatching::new_empty();
    //let mut agent_strat = CollisionFree::new();
    //agent_strat.prep(&map, &agents, &targets, &mut matcher);

    let mut runner = Runner{map: map.clone(), agents, targets, d_time};
    let took = runner.run(Box::new(agent_strat), &mut follow_path, false, true, true, false, "generated/debug_run.gif", 100);
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
