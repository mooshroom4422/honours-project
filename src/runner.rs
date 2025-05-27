use std::time::Instant;
use std::collections::HashSet;

use crate::map::*;
use crate::generate_gif::*;
use crate::agent_strategies::*;
use crate::target_strategies::*;
use log::{warn};

pub struct Runner {
    pub map: Map,
    pub agents: Vec<Agent>,
    pub targets: Vec<Target>,
    pub d_time: i32,
}

impl Runner {
    pub fn run(&mut self, mut agent_strat: Box<dyn AgentStrategy>, target_strat: &mut Box<dyn TargetStrategy>,
            debug_printing: bool, enable_runtime_checks: bool, enable_gif: bool, print_res: bool, gif_path: &str,
            MAX_ITER: i32) -> i32 {

        let start = Instant::now();
        let mut frames: Vec<Vec<u8>> = Vec::new();
        if debug_printing {
            println!("start:");
            print_board(&self.map, &self.agents, &self.targets);
        }

        let mut turns = 0;
        let mut iter = 0;
        while self.targets.len() > 0 && iter < MAX_ITER {
            iter += 1;
            if debug_printing { println!("========================"); }
            let target_dirs = target_strat.pick(&self.map, &self.agents, &self.targets);
            for (idx, dir) in target_dirs.iter().enumerate() {
                //println!("trying: {:?} {:?} at {}", self.targets[idx].position, *dir, iter);
                self.targets[idx].position = go_direction(self.targets[idx].position, *dir); // TODO: panics here
                if *dir == Direction::None {
                    self.targets[idx].timer = self.d_time;
                }
                else {
                    self.targets[idx].timer -= 1;
                }
            }

            let agent_dirs = agent_strat.pick(&self.map, &mut self.agents, &self.targets);
            for (idx, dir) in agent_dirs.iter().enumerate() {
                self.agents[idx].position = go_direction(self.agents[idx].position, *dir);
            }

            if enable_runtime_checks {
                // check if agent positions are unique

                // check if target positions are unique

                // check if all agents are on tiles (not on wall nor outside the grid)

            }

            // let mut used = HashSet::new();

            if debug_printing {
                println!("turn: {}", turns);
                println!("pre:");
                println!("{:?}", self.agents);
                println!("{:?}", self.targets);
            }

            let agent_positions = self.agents.clone()
                .into_iter()
                .filter(|x| x.active)
                .map(|x| (x.position, x.targets))
                .collect::<HashSet<_>>();

            let target_positions = self.targets.clone()
                .into_iter()
                .map(|x| x.position)
                .collect::<HashSet<_>>();

            for agent in &mut self.agents {
                if !agent.active {
                   // !target_positions.contains(&agent.position) ||
                   // used.contains(&agent.position) {
                    continue;
                }
                let same_pos = self.targets.clone()
                    .into_iter()
                    .filter(|x| x.position == agent.position)
                    .filter(|x| x.idx as i32 == agent.targets)
                    .collect::<Vec<_>>();
                if same_pos.is_empty() { continue; }
                // used.insert(agent.position);
                agent.active = false;
            }

            self.targets = self.targets.clone()
                .into_iter()
                .filter(|t| !(agent_positions.contains(&(t.position, t.idx as i32)) ||
                            agent_positions.contains(&(t.position, -1))))
                .collect::<Vec<_>>();

            if debug_printing {
                println!("post:");
                println!("{:?}", self.agents);
                println!("{:?}", self.targets);
            }

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
            // println!("helo");
            if got.is_err() {
                println!("error saving gif, make sure that '{}' directory is present", gif_path);
            }
        }

        if iter == MAX_ITER { warn!("max iter reached!"); }

        if self.targets.len() > 0 { println!("did not finish!"); }

        if print_res {
            println!("simulation took: {:?}", start.elapsed());
        }
        turns
    }
}

