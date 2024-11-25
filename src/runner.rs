use std::time::Instant;

use crate::map::*;
use crate::generate_gif::*;
use crate::agent_strategies::*;
use crate::target_strategies::*;

pub struct Runner {
    pub map: Map,
    pub agents: Vec<Agent>,
    pub targets: Vec<Target>,
    pub d_time: i32,
}

impl Runner {
    pub fn run(&mut self, mut agent_strat: impl AgentStrategy, mut target_strat: impl TargetStrategy,
            debug_printing: bool, enable_gif: bool, gif_path: &str) -> i32 {

        let start = Instant::now();
        let mut frames: Vec<Vec<u8>> = Vec::new();
        if debug_printing {
            println!("start:");
            print_board(&self.map, &self.agents, &self.targets);
        }

        let mut turns = 0;
        let mut iter = 0;
        while self.targets.len() > 0 && iter < 110 {
            iter += 1;
            if debug_printing { println!("========================"); }
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
            if debug_printing {
                println!("tg_di: {:?}", target_dirs);
                println!("ag_di: {:?}", agent_dirs);
                print_board(&self.map, &self.agents, &self.targets);
            }
            if enable_gif {
                let frame = generate_frame(&self.map, &self.agents, &self.targets);
                frames.push(frame);
            }
        }

        if enable_gif {
            let got = generate_gif(&frames, &self.map, gif_path);
            if got.is_err() {
                println!("error saving gif, make sure that '{}' directory is present", gif_path);
            }
        }

        println!("simulation took: {:?}", start.elapsed());
        turns
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
