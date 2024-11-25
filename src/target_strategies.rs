use rand::seq::SliceRandom;
use std::cmp;

use crate::map::*;

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

pub struct MaximizeMinDist;
impl TargetStrategy for MaximizeMinDist {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = vec![Direction::None; targets.len()];

        let dirs = Vec::from([Direction::North, Direction::East,
            Direction::South, Direction::West]);
        for (idx, target) in targets.iter().enumerate() {
            if target.timer == 0 {
                continue;
            }
            let mut best = usize::MAX;
            for agent in agents {
                best = cmp::min(best, map.dist_point(&agent.position, &target.position));
            }
            for dir in dirs.iter() {
                let pos = go_direction(target.position, *dir);
                if !map.valid_point(pos) { continue; }
                let mut now = usize::MAX;
                for agent in agents {
                    now = cmp::min(now, map.dist_point(&agent.position, &pos));
                }
                if now > best {
                    best = now;
                    res[idx] = *dir;
                }
            }
        }

        res
    }
}
