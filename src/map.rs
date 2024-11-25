use std::{fs, collections::VecDeque};
use std::cmp;

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
}

pub fn agents_from(points: &Vec<Point>) -> Vec<Agent> {
    let mut res = Vec::new();

    for p in points {
        res.push(Agent{position: *p});
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub fn go_direction(point: Point, direction: Direction) -> Point {
    match direction {
        Direction::North => Point{x: point.x-1, y: point.y},
        Direction::East => Point{x: point.x, y: point.y+1},
        Direction::South => Point{x: point.x+1, y: point.y},
        Direction::West => Point{x: point.x, y: point.y-1},
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

        let mut map = vec![vec![Tile::Wall; width]; height];
        for x in 0..height {
            for y in 0..width {
                if lines[x+4].as_bytes()[y] == b'.' {
                    map[x][y] = Tile::Free;
                }
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
        for stax in 0..height {
            for stay in 0..width {
                if !res.valid(stax, stay) { continue; }
                let mut q: VecDeque<(usize, usize)> = VecDeque::new();
                q.push_back((stax, stay));
                let stac = res.conv(stax, stay);
                dist[stac][stac] = 0;
                while !q.is_empty() {
                    let (x, y) = q.pop_front().unwrap();
                    let now = res.conv(x, y);

                    if x > 0 && res.valid(x-1, y) {
                        let to = res.conv(x-1, y);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::North;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x-1, y));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }

                    if x < height-1 && res.valid(x+1, y) {
                        let to = res.conv(x+1, y);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::South;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x+1, y));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }

                    if y > 0 && res.valid(x, y-1) {
                        let to = res.conv(x, y-1);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::West;
                            }
                            else {
                                from[stac][to] = from[stac][now].clone();
                            }
                            q.push_back((x, y-1));
                            dist[stac][to] = dist[stac][now]+1;
                        }
                    }

                    if y < width-1 && res.valid(x, y+1) {
                        let to = res.conv(x, y+1);
                        if dist[stac][to] == usize::MAX {
                            if from[stac][now] == Direction::None {
                                from[stac][to] = Direction::East;
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

    fn conv(&self, x: usize, y: usize) -> usize {
        return x*self.width+y;
    }

    fn valid(&self, x: usize, y: usize) -> bool {
        x < self.height && y < self.width && self.map[x][y] == Tile::Free
    }

    pub fn valid_direction(&self, p: Point, dir: Direction) -> bool {
        if p.x == 0 && dir == Direction::North { false }
        else if p.y == self.width && dir == Direction::East { false }
        else if p.x == self.height && dir == Direction::South { false }
        else if p.y == 0 && dir == Direction::West { false }
        else { true }
    }

    pub fn valid_point(&self, p: Point) -> bool {
        self.valid(p.x, p.y)
    }

    fn dist(&self, fx: usize, fy: usize, tx: usize, ty: usize) -> usize {
        if !self.valid(fx, fy) || !self.valid(tx, ty) {
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
        let mut got: Vec<Vec<bool>> = vec![vec![false; map.width]; map.height];
        for i in 0..map.height {
            for j in 0..map.width {
                got[i][j] = map.valid(i, j);
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
