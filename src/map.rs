use std::{fs, collections::{VecDeque, HashSet}};
use std::cmp;
use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
enum Tile {
    Wall,
    Free,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
    None,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Agent {
    pub position: Point,
    pub active: bool,
    pub targets: i32, // -1 -> any
}

impl Agent {
    fn from(point: Point) -> Agent {
        Agent {
            position: point,
            active: true,
            targets: -1,
        }
    }
}

pub fn agents_from(points: &Vec<Point>) -> Vec<Agent> {
    let mut res = Vec::new();

    for p in points {
        res.push(Agent::from(*p));
    }

    res
}

pub fn agents_random(map: &Map, n: usize) -> Vec<Agent> {
    let mut res = Vec::new();
    let mut rng = rand::thread_rng();

    let mut points_taken = HashSet::new();

    for _ in 0..n {
        loop {
            let x = rng.gen_range(0..map.width);
            let y = rng.gen_range(0..map.height);
            if !points_taken.contains(&Point{x, y}) &&
               map.valid_point(&Point{x, y}) {
                res.push(Agent::from(Point{x, y}));
                points_taken.insert(Point{x, y});
                break;
            }
        }
    }

    res
}

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub position: Point,
    pub timer: i32,
    pub path: Option<Vec<Point>>,
    pub idx: usize,
}

impl Target {
    // remember about timer: d
    pub fn at_time(&self, time: usize) -> Point {
        assert!(self.path.is_some());
        let path: &Vec<Point> = self.path.as_ref().unwrap();
        path[cmp::min(time, path.len()-1)].clone()
    }
}

pub fn targets_from(points: &Vec<Point>, timer: i32) -> Vec<Target> {
    let mut res = Vec::new();

    for p in points {
        res.push(Target{position: *p, timer, path: None, idx: 0});
    }

    res
}

pub fn targets_random(map: &Map, n: usize, timer: i32) -> Vec<Target> {
    let mut res = Vec::new();
    let mut rng = rand::thread_rng();

    for idx in 0..n {
        loop {
            let x = rng.gen_range(0..map.width);
            let y = rng.gen_range(0..map.height);
            if !res.contains(&Target{ idx, position: Point{x, y}, timer, path: None }) &&
                map.valid_point(&Point{x, y}) {
                res.push(Target{ idx, position: Point{x, y}, timer, path: None });
                break;
            }
        }
    }

    res
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub fn go_direction(point: Point, direction: Direction) -> Point {
    match direction {
        Direction::North => Point{x: point.x, y: point.y+1},
        Direction::East => Point{x: point.x+1, y: point.y},
        Direction::South => Point{x: point.x, y: point.y-1},
        Direction::West => Point{x: point.x-1, y: point.y},
        Direction::None => Point{x: point.x, y: point.y},
    }
}

#[derive(Clone)]
pub struct Map {
    pub height: usize,
    pub width: usize,
    map: Vec<Vec<Tile>>,
    dist: Vec<Vec<usize>>,
    from: Vec<Vec<Direction>>,
}

impl Map {
    pub fn new(file_path: &str) -> Self {
        let file = fs::read_to_string(file_path)
            .expect("error reading file");

        let lines = file.lines()
            .into_iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();

        // skip type for now
        let height = lines[1].split(" ")
            .collect::<Vec<_>>()[1]
            .parse::<usize>().unwrap();

        let width = lines[2].split(" ")
            .collect::<Vec<_>>()[1]
            .parse::<usize>().unwrap();

        let mut map = vec![vec![Tile::Wall; height]; width];
        for y in (0..height).rev() {
            for x in 0..width {
                if lines[(height-y-1)+4].as_bytes()[x] == b'.' {
                    map[x][y] = Tile::Free;
                }
                // println!("x={} y={} = {:?}", x, y, map[x][y]);
            }
        }

        // for now, precompute distances using bfs
        let mut res = Map {
            height,
            width,
            map,
            dist: Vec::new(),
            from: Vec::new(),
        };

        let tiles = height*width;
        let mut dist = vec![vec![usize::MAX; tiles]; tiles];
        let mut from = vec![vec![Direction::None; tiles]; tiles];
        for stax in 0..width {
            for stay in 0..height {
                if !res.valid_point_expl(stax, stay) { continue; }
                let mut q: VecDeque<(usize, usize)> = VecDeque::new();
                q.push_back((stax, stay));
                let stac = res.conv(stax, stay);
                dist[stac][stac] = 0;
                while !q.is_empty() {
                    let (x, y) = q.pop_front().unwrap();
                    let now = res.conv(x, y);

                    if x > 0 && res.valid_point_expl(x-1, y) {
                        let to = res.conv(x-1, y);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::West;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x-1, y));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }

                    if res.valid_point_expl(x+1, y) {
                        let to = res.conv(x+1, y);
                        // println!("failed: x={}, y={}, conv={}", x, y, to);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::East;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x+1, y));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }

                    if y > 0 && res.valid_point_expl(x, y-1) {
                        let to = res.conv(x, y-1);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::South;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x, y-1));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }

                    if res.valid_point_expl(x, y+1) {
                        let to = res.conv(x, y+1);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::North;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x, y+1));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }
               }
            }
        }

        res.dist = dist;
        res.from = from;

        res
    }

    pub fn conv(&self, x: usize, y: usize) -> usize {
        return y*self.width+x;
    }

    fn valid_point_expl(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && self.map[x][y] == Tile::Free
    }

    pub fn reverse_direction(dir: &Direction) -> Direction {
        match dir {
            Direction::None => Direction::None,
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    pub fn valid_direction(&self, p: Point, dir: Direction) -> bool {
        if p.y == self.height && dir == Direction::North { false }
        else if p.x == self.width && dir == Direction::East { false }
        else if p.y == 0 && dir == Direction::South { false }
        else if p.x == 0 && dir == Direction::West { false }
        else { true }
    }

    pub fn valid_point(&self, p: &Point) -> bool {
        self.valid_point_expl(p.x, p.y)
    }

    fn dist(&self, fx: usize, fy: usize, tx: usize, ty: usize) -> usize {
        if !self.valid_point_expl(fx, fy) || !self.valid_point_expl(tx, ty) {
            return usize::MAX;
        }
        return self.dist[self.conv(fx, fy)][self.conv(tx, ty)];
    }

    pub fn dist_point(&self, p1: &Point, p2: &Point) -> usize {
        self.dist(p1.x, p1.y, p2.x, p2.y)
    }

    fn get_dist(&self, x: usize, y: usize) -> &Vec<usize> {
        &self.dist[self.conv(x, y)]
    }

    pub fn get_direction(&self, p1: &Point, p2: &Point) -> Direction {
        self.from[self.conv(p1.x, p1.y)][self.conv(p2.x, p2.y)]
    }
}

pub fn print_board(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) {
    for y in (0..map.height).rev() {
        for x in 0..map.width {
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
            else if map.valid_point(&Point{x, y}){
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let map = Map::new("resources/maps/example.map");
        let exp: Vec<Vec<Tile>> = Vec::from([
                Vec::from([Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall]),
                Vec::from([Tile::Wall, Tile::Free, Tile::Free, Tile::Free, Tile::Wall]),
                Vec::from([Tile::Wall, Tile::Free, Tile::Wall, Tile::Free, Tile::Wall]),
                Vec::from([Tile::Wall, Tile::Free, Tile::Free, Tile::Free, Tile::Wall]),
                Vec::from([Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall]),
            ]);
        assert_eq!(exp, map.map);
    }

    #[test]
    fn test_valid() {
        let map = Map::new("resources/maps/example.map");
        let mut got: Vec<Vec<bool>> = vec![vec![false; map.height]; map.width];
        for i in 0..map.height {
            for j in 0..map.width {
                got[i][j] = map.valid_point_expl(i, j);
            }
        }

        let exp: Vec<Vec<bool>> = Vec::from([
                Vec::from([false, false, false, false, false]),
                Vec::from([false, true, true, true, false]),
                Vec::from([false, true, false, true, false]),
                Vec::from([false, true, true, true, false]),
                Vec::from([false, false, false, false, false]),
            ]);

        assert_eq!(exp, got);
    }

    #[test]
    fn test_dist() {
        let map = Map::new("resources/maps/example.map");
        let mut exp: Vec<usize> = Vec::from([
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
            ]);
        assert_eq!(exp, *map.get_dist(0, 0));

        exp = Vec::from([
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, 0, 1, 2, usize::MAX,
                usize::MAX, 1, usize::MAX, 3, usize::MAX,
                usize::MAX, 2, 3, 4, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
            ]);
        assert_eq!(exp, *map.get_dist(1, 1));

        exp = Vec::from([
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, 3, 4, 3, usize::MAX,
                usize::MAX, 2, usize::MAX, 2, usize::MAX,
                usize::MAX, 1, 0, 1, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
            ]);
        assert_eq!(exp, *map.get_dist(3, 2));
    }

    #[test]
    fn test_from() {
        let map = Map::new("resources/maps/example.map");

        let exp: Vec<Direction> = Vec::from([
                Direction::None, Direction::None, Direction::None, Direction::None, Direction::None,
                Direction::None, Direction::None, Direction::East, Direction::East, Direction::None,
                Direction::None, Direction::South, Direction::None, Direction::East, Direction::None,
                Direction::None, Direction::South, Direction::South, Direction::South, Direction::None,
                Direction::None, Direction::None, Direction::None, Direction::None, Direction::None,
            ]);

        assert_eq!(exp, map.from[map.conv(1, 1)]);
    }

}
