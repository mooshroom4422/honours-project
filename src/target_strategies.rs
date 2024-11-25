use rand::seq::SliceRandom;
use std::collections::HashSet;

use crate::map::*;

pub trait TargetStrategy {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub struct RandomTarget;
impl TargetStrategy for RandomTarget {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
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
            if iter == 20 { dir = &Direction::None; }
            res[idx] = *dir;
        }

        res
    }
}

pub struct TargetFollowPath {
    paths: Vec<Vec<Direction>>,
    path_idx: Vec<usize>,
    starting_points: Vec<Point>,
    blocked: Vec<Vec<HashSet<usize>>>,
}

impl TargetFollowPath {
    pub fn new(n: usize,  map: &Map, starting_points: Vec<Point>, targets: &mut Vec<Target>,
               generate: bool, len: i32) -> Self {
        let mut res = TargetFollowPath{
            paths: Vec::new(),
            path_idx: vec![0; n],
            starting_points: Vec::new(),
            blocked: Vec::new()
        };
        res.create(n, map, starting_points, targets, generate, len);
        res
    }

    fn create(&mut self, n: usize, map: &Map, starting_points: Vec<Point>,
              targets: &mut Vec<Target>, generate: bool, len: i32) {
        self.paths = vec![Vec::new(); n];
        self.blocked = vec![vec![HashSet::new(); map.height]; map.width];
        assert!(starting_points.len() == n);
        self.starting_points = starting_points;
        for point in self.starting_points.iter() {
            assert!(!self.blocked[point.x][point.y].contains(&0));
            self.blocked[point.x][point.y].insert(0);
        }
        if !generate { return; }
        for (i, target) in targets.iter_mut().enumerate() {
            self.generate_path(i, len, map, self.starting_points[i], target.timer);
            self.generate_path_target(map, i, self.starting_points[i], target);
        }
        //println!("{:?}", targets);
    }

    fn is_blocked(&self, pt: &Point, time: usize) -> bool {
        let x = pt.x;
        let y = pt.y;
        self.blocked[x][y].contains(&time)
    }

    fn generate_path(&mut self, idx: usize, mut len: i32, map: &Map, start_position: Point, timer: i32) {
        if len == -1 {
            len = 10; // randomize later
        }

        let dirs = Vec::from([Direction::North, Direction::East,
            Direction::South, Direction::West, Direction::None]);
        let mut position = start_position.clone();
        let mut time_now = timer;
        for i in 0..len {
            let mut iter = 0;
            let mut dir = dirs.choose(&mut rand::thread_rng()).unwrap();
            if time_now == 0 { iter = std::i32::MAX; }
            while iter < 20 && (!map.valid_point(go_direction(position, *dir))
                || self.is_blocked(&go_direction(position, *dir), (i+1) as usize)) {

                dir = dirs.choose(&mut rand::thread_rng()).unwrap();
                iter += 1;
            }
            if iter >= 20 { dir = &Direction::None; }
            // TODO: debug?
            if !map.valid_point(go_direction(position, *dir)) {
                println!("{:?}, {:?}, {:?}", position, *dir, iter);
                panic!();
            }
            if *dir == Direction::None { time_now = timer; }
            else { time_now -= 1; }
            self.paths[idx].push(*dir);
            position = go_direction(position, *dir);
            self.blocked[position.x][position.y].insert((i+1) as usize);
        }
    }

    fn generate_path_target(&mut self, map: &Map, idx: usize, starting_position: Point, target: &mut Target) {
        let mut res: Vec<Point> = Vec::new();
        res.push(starting_position);
        let mut position_now = starting_position;
        let d = target.timer;
        loop {
            if self.path_idx[idx] >= self.paths[idx].len() {
                break;
            }
            if target.timer == 0 {
                target.timer = d;
                continue;
            }
            position_now = go_direction(position_now, self.paths[idx][self.path_idx[idx]]);
            // TODO: debug?
            if !map.valid_point(position_now) {
                println!("{:?}, {:?}", position_now, self.path_idx[idx]);
                panic!();
            }
            res.push(position_now);
            self.path_idx[idx] += 1;
            target.timer -= 1;
        }
        self.path_idx[idx] = 0;
        target.timer = d;

        target.path = Some(res);
        target.idx = idx;
    }

    pub fn set_path(&mut self, idx: usize, path: &Vec<Direction>, map: &Map,
                    target: &mut Target, account_for_d: bool) {
        if account_for_d { unimplemented!(); }
        let mut cells = Vec::new();
        let mut now = target.position.clone();
        cells.push(now);
        for dir in path.iter() {
            now = go_direction(now, *dir);
            cells.push(now);
        }
        target.path = Some(cells);
        self.paths[idx] = path.clone();
    }
}

impl TargetStrategy for TargetFollowPath {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = Vec::new();

        for target in targets.iter() {
            let idx = target.idx;
            if self.path_idx[idx] >= self.paths[idx].len() {
                res.push(Direction::None);
                continue;
            }
            if target.timer == 0 && self.paths[idx][self.path_idx[idx]] != Direction::None {
                // this condition should be checked by path generation algorithm
                panic!("if timer is 0 direction has to be none!");
            }
            res.push(self.paths[idx][self.path_idx[idx]]);
            self.path_idx[idx] += 1;
        }

        res
    }
}
