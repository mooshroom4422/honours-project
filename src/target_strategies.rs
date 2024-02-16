use rand::seq::SliceRandom;

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
