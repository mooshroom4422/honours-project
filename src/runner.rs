use rand::seq::SliceRandom;

use crate::hopcroft_karp::HopcroftKarp;
use crate::map::*;
use crate::matching::{Matcher, makespan_solve};

pub trait AgentStrategy {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub struct MakeSpanHopcroft;
impl AgentStrategy for MakeSpanHopcroft {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = vec![Direction::None; agents.len()];

        let agents_point = agents.into_iter()
            .map(|x| x.position)
            .collect::<Vec<_>>();

        let targets_point = targets.into_iter()
            .map(|x| x.position)
            .collect::<Vec<_>>();

        let mut matcher = HopcroftKarp::new_empty();
        let dd = makespan_solve(map, &agents_point, &targets_point, &mut matcher);

        let matching = matcher.get_matching();
        println!("dd: {:?}", dd);
        println!("matching: {:?}", matching);
        for i in 0..agents.len() {
            if matching[i] == -1 { continue; }
            let t = (matching[i] as usize)-agents.len();
            res[i] = map.get_direction(&agents[i].position, &targets[t].position);
        }

        res
    }
}

pub trait TargetStrategy {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub struct RandomTarget;
impl TargetStrategy for RandomTarget {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = vec![Direction::None; targets.len()];

        let dirs = Vec::from([Direction::North, Direction::East,
            Direction::South, Direction::West, Direction::None]);
        for (idx, target) in targets.iter().enumerate() {
            if target.timer == 0 {
                continue;
            }
            let mut iter = 0;
            let mut dir = dirs.choose(&mut rand::thread_rng()).unwrap();
            while iter < 20 && !map.valid_point(go_direction(target.position, *dir)) {
                dir = dirs.choose(&mut rand::thread_rng()).unwrap();
                iter += 1;
            }
            res[idx] = *dir;
        }

        res
    }
}

pub struct Runner {
    pub map: Map,
    pub agents: Vec<Agent>,
    pub targets: Vec<Target>,
    pub d_time: i32,
}

impl Runner {
    pub fn run(&mut self, agent_strat: impl AgentStrategy, target_strat: impl TargetStrategy) {
        println!("start:");
        print_board(&self.map, &self.agents, &self.targets);
        let mut turns = 0;
        while self.targets.len() > 0 {
            println!("========================");
            let target_dirs = target_strat.pick(&self.map, &self.agents, &self.targets);
            for (idx, dir) in target_dirs.iter().enumerate() {
                self.targets[idx].position = go_direction(self.targets[idx].position, *dir);
                if *dir == Direction::None {
                    self.targets[idx].timer = self.d_time;
                }
                else {
                    self.targets[idx].timer -= 1;
                }
            }

            let agent_dirs = agent_strat.pick(&self.map, &self.agents, &self.targets);
            for (idx, dir) in agent_dirs.iter().enumerate() {
                self.agents[idx].position = go_direction(self.agents[idx].position, *dir);
            }

            self.targets = self.targets.clone()
                .into_iter()
                .filter(|t| !self.agents.contains(&Agent{position: t.position}))
                .collect::<Vec<_>>();

            turns += 1;
            // debug
            println!("tg_di: {:?}", target_dirs);
            println!("ag_di: {:?}", agent_dirs);
            print_board(&self.map, &self.agents, &self.targets);
        }
        println!("took: {}", turns);
    }
}

fn print_board(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) {
    for x in 0..map.height {
        for y in 0..map.height {
            let ag = agents.into_iter()
                .any(|f| f.position == Point{x, y});
            let tr = targets.into_iter()
                .any(|f| f.position == Point{x, y});
            if ag && tr {
                print!("F");
            }
            else if ag {
                print!("A");
            }
            else if tr {
                print!("T");
            }
            else if map.valid_point(Point{x, y}){
                print!(".");
            }
            else {
                print!("X");
            }
        }
        print!("\n");
    }
    println!("agents: {:?}", agents);
    println!("targets: {:?}", targets);
}
