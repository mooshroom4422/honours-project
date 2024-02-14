use rand::seq::SliceRandom;

use crate::map::*;

pub trait AgentStrategy {
    fn pick(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub trait TargetStrategy {
    fn pick(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

struct RandomTarget;
impl TargetStrategy for RandomTarget {
    fn pick(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = vec![Direction::None; agents.len()];

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
    map: Map,
    agents: Vec<Agent>,
    targets: Vec<Target>,
    d_time: i32,
}

impl Runner {
    pub fn run(&mut self, agent_strat: impl AgentStrategy, target_strat: impl TargetStrategy) {
        while self.targets.len() > 0 {



            self.targets = self.targets.clone()
                .into_iter()
                .filter(|t| self.agents.contains(&Agent{position: t.position}))
                .collect::<Vec<_>>();
        }
    }
}
