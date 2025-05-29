use std::{fs, collections::{VecDeque, HashSet, HashMap}};
use std::cmp;
use rand::Rng;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, Write, Read};
use bytesize::ByteSize;
use log::{info, trace};
use tqdm::tqdm;

#[derive(Clone, Debug, PartialEq)]
enum Tile {
    Wall,
    Free,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
    None,
    Unreachable,
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
        Direction::Unreachable => panic!(),
    }
}

// #[derive(Clone)]
// avoid clonning the maps struct since it contains the distance oracle
pub struct Map {
    pub height: usize,
    pub width: usize,
    map: Vec<Vec<Tile>>,
    compressed: Vec<Vec<Vec<Rect>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Rect {
    xd: usize,
    yd: usize,
    xu: usize,
    yu: usize,
    dir: Direction,
}

impl Map {
    const DIRS: [(Direction, (i32, i32)); 5] = [
        (Direction::North, ( 0,  1)),
        (Direction::East,  ( 1,  0)),
        (Direction::South, ( 0, -1)),
        (Direction::West,  (-1,  0)),
        (Direction::None,  ( 0,  0)),
    ];

    pub fn new(file_path: &str) -> Self {
        let start = Instant::now();

        info!("reading: {}", file_path);
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

        // make sure that the borders are walls
        for y in 0..height {
            map[0][y] = Tile::Wall;
            map[width-1][y] = Tile::Wall;
        }

        for x in 0..width {
            map[x][0] = Tile::Wall;
            map[x][height-1] = Tile::Wall;
        }

        let database_path = format!("{}.dist", file_path);
        let got = Map::load(&database_path);

        if let Some(compressed) = got {
            info!("loaded: {}", database_path);
            let size = compressed.iter()
                .flat_map(|v| v.iter())
                .map(|v| v.len())
                .sum::<usize>() * size_of::<Rect>() / (1024*1024);
            info!("in memory size: {:.2}MB", size);
            trace!("width: {}", width);
            trace!("height: {}", height);
            trace!("compressed.len(): {}", compressed.len());
            trace!("compressed[0].len(): {}", compressed[0].len());
            return Map {
                height,
                width,
                map,
                compressed,
            };
        }
        else {
            info!(".dist file not found, starting calculation");
        }

        let mut res = Map {
            height,
            width,
            map,
            // dist: Vec::new(),
            // from: Vec::new(),
            compressed: Vec::new(),
        };

        // print_board(&res, &Vec::new(), &Vec::new());

        let mut normal_size = 0;
        let mut compressed_size = 0;

        let mut compressed = vec![vec![Vec::new(); height]; width];

        for y in tqdm(0..height) {
            for x in 0..width {
                if res.valid_point_expl(x, y) {
                    let (initial_rectagle, prefs) = Map::compute_board(&mut res, x as i32, y as i32);
                    //println!("{:?}, {:?}", initial_rectagle, prefs);
                    let got = Map::compress(initial_rectagle, prefs);
                    // if x == 24 && y == 3 {
                    //     println!("{:?}", got);
                    //     //panic!();
                    // }
                    // size of n^2 lookup table, a bit optimistic
                    // *2, since we keep both direction and distance
                    normal_size += (height-2)*(width-2)*2;
                    // *5, since we keep 2 points and direction in a Rect struct
                    compressed_size += got.len()*3;
                    compressed[x][y] = got;
                    //println!("{}, {}: \n {:?}", x, y, compressed.len());
                }
            }
            // trace!("finished y: {}/{}, completed: {:.2}%", y, height, (y as f64)/(height as f64)*100.0);
        }

        let ratio = (normal_size as f64)/(compressed_size as f64);
        trace!("normal: {}", normal_size);
        trace!("compressed: {}", compressed_size);
        info!("compression ratio: {}", ratio);
        info!("compression took: {:?}", start.elapsed());

        let serialized_data = bincode::serialize(&compressed).expect("failed to serialize data");

        let mut file = File::create(&database_path)
            .expect("failed to create file: {}");
        file.write_all(&serialized_data)
            .expect("failed to write to file");

        info!("saved dist to: {}", database_path);
        info!("file size: {}", ByteSize(serialized_data.len() as u64));

        res.compressed = compressed;

        res
    }

    // assuming (stax, stay) is walkable
    fn compute_board(map: &mut Map, stax: i32, stay: i32) -> (Rect, HashMap<Direction, Vec<Vec<i32>>>) {
        let mut q: VecDeque<(i32, i32)> = VecDeque::new();
        let mut from: Vec<Vec<Direction>> = vec![vec![Direction::Unreachable; map.height]; map.width];

        for (dir, (dx, dy)) in Map::DIRS.iter() {
            let x = stax + dx;
            let y = stay + dy;
            if map.valid_point_i32(x, y) {
                q.push_back((x, y));
                from[x as usize][y as usize] = *dir;
            }
        }

        // if stax == 16 && stay == 6 {
        //     info!("from");
        //     print_from(map, &from);
        // }

        while !q.is_empty() {
            let (px, py) = q.pop_front().unwrap();

            for (dir, (dx, dy)) in Map::DIRS.iter() {
                let x = px + dx;
                let y = py + dy;
                if map.valid_point_i32(x, y) && from[x as usize][y as usize] == Direction::Unreachable {
                    q.push_back((x, y));
                    from[x as usize][y as usize] = from[px as usize][py as usize];
                }
            }
        }

        // if stax == 16 && stay == 6 {
        //     info!("from");
        //     print_from(map, &from);
        // }

        let mut pref = HashMap::new();

        for (dir, _) in Map::DIRS.iter() {
            let mut pr = vec![vec![0; map.height]; map.width];
            for x in 0..map.width {
                if from[x][0] == *dir {
                    pr[x][0] = 1;
                }
            }
            for y in 1..map.height {
                if from[0][y] == *dir {
                    pr[0][y] = 1;
                }
            }

            for x in 1..map.width {
                for y in 1..map.height {
                    pr[x][y] = pr[x][y-1] + pr[x-1][y] - pr[x-1][y-1];
                    if from[x][y] == *dir {
                        pr[x][y] += 1;
                    }
                }
            }
            pref.insert(*dir, pr);
        }

        // println!("from: {:?}", from);
        //println!("finished (from): {} {}", stax, stay);
        //print_from(map, &from);
        //panic!();

        // assuming that the map is surrounded by walls -> WRONG! it is not always the case, double check each map
        let mut rect = Rect { xd: 1, yd: 1, xu: map.width-2, yu: map.height-2, dir: Direction::Unreachable };

        (rect, pref)
    }

    fn pref_get(pref: &Vec<Vec<i32>>, xd: i32, yd: i32, xu: i32, yu: i32) -> i32 {
        let mut res = pref[xu as usize][yu as usize];
        if xd > 0 { res -= pref[(xd-1) as usize][yu as usize]; }
        if yd > 0 { res -= pref[xu as usize][(yd-1) as usize]; }
        if xd > 0 && yd > 0 { res += pref[(xd-1) as usize][(yd-1) as usize]; }
        res
    }

    fn compress(initial_rect: Rect, pref: HashMap<Direction, Vec<Vec<i32>>>) -> Vec<Rect> {
        let mut res: Vec<Rect> = Vec::new();
        let mut queue: VecDeque<Rect> = VecDeque::new();

        queue.push_back(initial_rect);

        while !queue.is_empty() {
            // println!("================");
            // println!("res: {:?}", res);
            let mut rect = queue.pop_front().unwrap();
            // println!("{:?}", rect);

            if rect.xu < rect.xd || rect.yu < rect.yd {
                continue;
            }

            if rect.xu == rect.xd && rect.yu == rect.yd {
                for (dir, _) in Map::DIRS.iter() {
                    let pr = pref.get(dir).unwrap();
                    if Map::pref_get(pr, rect.xd as i32, rect.yd as i32, rect.xu as i32, rect.yu as i32) == 1 {
                        rect.dir = *dir;
                        res.push(rect);
                        break;
                    }
                }
                continue;
            }

            let mut splits: Vec<(i32, Rect, Option<Rect>)> = Vec::new();

            // if pref dir == sum pref -> uni
            for k in rect.xd .. rect.xu+1 {
                // +----------+ yu
                // |    |     |
                // |    |     |
                // |    |     |
                // +----------+ yd
                // xd   k     xu
                let mut left = Vec::new();
                let mut right = Vec::new();

                for (dir, _) in Map::DIRS.iter() {
                    let pr = pref.get(dir).unwrap();
                    left.push((*dir, Map::pref_get(pr, rect.xd as i32, rect.yd as i32, k as i32, rect.yu as i32)));
                    right.push((*dir, Map::pref_get(pr, k as i32, rect.yd as i32, rect.xu as i32, rect.yu as i32)));
                }

                left.sort_by(|l, r| r.1.cmp(&l.1));
                right.sort_by(|l, r| r.1.cmp(&l.1));

                if left[0].1 > 0 && left[1].1 == 0 {
                    let sz = (k-rect.xd+1)*(rect.yu-rect.yd+1);
                    let homo = Rect { xd: rect.xd, yd: rect.yd, xu: k, yu: rect.yu, dir: left[0].0 };
                    let rest = Rect { xd: k+1, yd: rect.yd, xu: rect.xu, yu: rect.yu, dir: Direction::Unreachable };
                    splits.push((sz as i32, homo, Some(rest)));
                }

                if right[0].1 > 0 && right[1].1 == 0 {
                    if k > 0 {
                        let sz = (k-rect.xd+1)*(rect.yu-rect.yd+1);
                        let homo = Rect { xd: k, yd: rect.yd, xu: rect.xu, yu: rect.yu, dir: right[0].0 };
                        let rest = Rect { xd: rect.xd, yd: rect.yd, xu: k-1, yu: rect.yu, dir: Direction::Unreachable };
                        splits.push((sz as i32, homo, Some(rest)));
                    }
                    else {
                        let sz = (k-rect.xd+1)*(rect.yu-rect.yd+1);
                        let homo = Rect { xd: k, yd: rect.yd, xu: rect.xu, yu: rect.yu, dir: right[0].0 };
                        splits.push((sz as i32, homo, None));
                    }
                }
            }

            for k in rect.yd .. rect.yu+1 {
                // +----------+ yu
                // |          |
                // |----------| k
                // |          |
                // +----------+ yd
                // xd         xu

                let mut down = Vec::new();
                let mut up = Vec::new();

                for (dir, _) in Map::DIRS.iter() {
                    let pr = pref.get(dir).unwrap();
                    down.push((*dir, Map::pref_get(pr, rect.xd as i32, rect.yd as i32, rect.xu as i32, k as i32)));
                    up.push((*dir, Map::pref_get(pr, rect.xd as i32, k as i32, rect.xu as i32, rect.yu as i32)));
                }

                down.sort_by(|l, r| r.1.cmp(&l.1));
                up.sort_by(|l, r| r.1.cmp(&l.1));

                //println!("up: {:?}", up);
                //println!("down: {:?}", down);

                if down[0].1 > 0 && down[1].1 == 0 {
                    let sz = (rect.xu-rect.xd+1)*(k-rect.yd+1);
                    let homo = Rect { xd: rect.xd, yd: rect.yd, xu: rect.xu, yu: k, dir: down[0].0 };
                    let rest = Rect { xd: rect.xd, yd: k+1, xu: rect.xu, yu: rect.yu, dir: Direction::Unreachable };
                    splits.push((sz as i32, homo, Some(rest)));
                }

                if up[0].1 > 0 && up[1].1 == 0 {
                    if k > 0 {
                        let sz = (rect.xu-rect.xd+1)*(rect.yu-k+1);
                        let homo = Rect { xd: rect.xd, yd: k, xu: rect.xu, yu: rect.yu, dir: up[0].0 };
                        let rest = Rect { xd: rect.xd, yd: rect.yd, xu: rect.xu, yu: k-1, dir: Direction::Unreachable };
                        splits.push((sz as i32, homo, Some(rest)));
                    }
                    else {
                        let sz = (rect.xu-rect.xd+1)*(rect.yu-k+1);
                        let homo = Rect { xd: rect.xd, yd: k, xu: rect.xu, yu: rect.yu, dir: up[0].0 };
                        splits.push((sz as i32, homo, None));
                    }
                }
            }

            //println!("splits: {:?}", splits);

            if splits.is_empty() {
                // if no homogeneous splits are possible split by dimensions
                let xl = rect.xu-rect.xd;
                let yl = rect.yu-rect.yd;
                if yl < xl {
                    // split on x
                    queue.push_back(Rect { xd: rect.xd, yd: rect.yd, xu: rect.xd+xl/2, yu: rect.yu, dir: Direction::Unreachable} );
                    queue.push_back(Rect { xd: rect.xd+xl/2+1, yd: rect.yd, xu: rect.xu, yu: rect.yu, dir: Direction::Unreachable} );
                }
                else {
                    // split on y
                    queue.push_back(Rect { xd: rect.xd, yd: rect.yd, xu: rect.xu, yu: rect.yd+yl/2, dir: Direction::Unreachable} );
                    queue.push_back(Rect { xd: rect.xd, yd: rect.yd+yl/2+1, xu: rect.xu, yu: rect.yu, dir: Direction::Unreachable} );
                }
            }
            else {
                // choose the best split
                splits.sort_by(|l, r| r.0.cmp(&l.0));
                res.push(splits[0].1.clone());
                if let Some(rest) = splits[0].2.clone() {
                    queue.push_back(rest);
                }
            }
        }

        res
    }

    fn load(file_path: &str) -> Option< Vec<Vec<Vec<Rect>>> > {
        info!("loading the .dist file: {}", file_path);
        let mut file = File::open(file_path).ok()?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("failed to read from file");

        let deserialized: Vec<Vec<Vec<Rect>>> = bincode::deserialize(&buffer)
            .expect("failed to deserialize");

        info!("finished");

        Some(deserialized)
    }

    pub fn conv(&self, x: usize, y: usize) -> usize {
        return y*self.width+x;
    }

    fn valid_point_expl(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && self.map[x][y] == Tile::Free
    }

    fn valid_point_i32(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 &&
        x < (self.width as i32) && y < (self.height as i32) &&
        self.map[x as usize][y as usize] == Tile::Free
    }

    pub fn reverse_direction(dir: &Direction) -> Direction {
        match dir {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::None => Direction::None,
            Direction::Unreachable => Direction::Unreachable,
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
        let mut res = 0;
        let pto = Point { x: tx, y: ty };
        let mut now = Point { x: fx, y: fy };
        // println!("calling dist: ({}, {}), ({}, {})", now.x, now.y, tx, ty);
        // println!("{:?} -> {:?}", self.map[now.x][now.y], self.map[tx][ty]);

        // info!("compressed:");
        // self.print_compressed_from(&Point{x:24, y:3});
        // println!("{:?}", self.compressed[24][3]);
        // panic!();

        while now.x != tx || now.y != ty {
            res += 1;
            let dir = self.get_direction(&now, &pto);
            now = go_direction(now, dir);
            //println!("({} {}) ({} {}), {:?}", now.x, now.y, tx, ty, dir);
            //println!("{:?}", now);
        }
        //println!("got: {}", res);
        return res;
    }

    pub fn dist_point(&self, p1: &Point, p2: &Point) -> usize {
        self.dist(p1.x, p1.y, p2.x, p2.y)
    }

    //fn get_dist(&self, x: usize, y: usize) -> &Vec<usize> {
    //    &self.dist[self.conv(x, y)]
    //}

    fn inside(rect: &Rect, p: &Point) -> bool {
        return rect.xd <= p.x && p.x <= rect.xu &&
               rect.yd <= p.y && p.y <= rect.yu
    }

    pub fn get_direction(&self, p1: &Point, p2: &Point) -> Direction {
        //self.from[self.conv(p1.x, p1.y)][self.conv(p2.x, p2.y)]
        // println!("========");
        // println!("{:?} -> {:?}", self.map[p1.x][p1.y], self.map[p2.x][p2.y]);
        // println!("{:?}", self.compressed[p1.x][p1.y]);
        // println!("{:?} -> {:?}", p1, p2);
        for rect in self.compressed[p1.x][p1.y].iter() {
            if Map::inside(&rect, &p2) {
                // println!("returning: {:?}", rect.dir);
                return rect.dir;
            }
        }
        panic!("no direction was found!")
    }

    pub fn print_compressed_from(&self, pnt: &Point) {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let p2 = Point{x, y};
                if !self.valid_point(&p2) {
                    print!("{:>2}", elegant(&Direction::Unreachable ));
                }
                else {
                    print!("{:>2}", elegant(&self.get_direction(pnt, &p2)));
                }
            }
            print!("\n");
        }
    }

    pub fn neighbor(&self, p1: &Point, p2: &Point) -> Direction {
        let dist = p1.x.abs_diff(p2.x) + p1.y.abs_diff(p2.y);
        assert!(dist <= 1, "{:?} -> {:?}", p1, p2);
        if dist == 0 {
            return Direction::None;
        }
        else if p1.x < p2.x {
            return Direction::East;
        }
        else if p2.x < p1.x {
            return Direction::West;
        }
        else if p1.y < p2.y {
            return Direction::North;
        }
        else {
            return Direction::South;
        }
    }
}

pub fn elegant(dir: &Direction) -> String {
    match dir {
        Direction::North => "↑".to_string(),
        Direction::East => "→".to_string(),
        Direction::South => "↓".to_string(),
        Direction::West => "←".to_string(),
        Direction::None => "O".to_string(),
        Direction::Unreachable => "X".to_string(),
    }
}

pub fn print_from(map: &Map, from: &Vec<Vec<Direction>>) {
    for y in (0..map.height).rev() {
        for x in 0..map.width {
            print!("{:>2}", elegant(&from[x][y]));
        }
        print!("\n");
    }
}

pub fn print_board(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) {
    print!("{:>5}", "");
    for x in 0..map.width {
        print!("{:>3}", x);
    }
    print!("\n");
    for y in (0..map.height).rev() {
        print!("{:>5}", y);
        for x in 0..map.width {
            let ag = agents.into_iter()
                .any(|f| f.position == Point{x, y});
            let tr = targets.into_iter()
                .any(|f| f.position == Point{x, y});
            if ag && tr {
                print!("{:>3}", "F");
            }
            else if ag {
                print!("{:>3}", "A");
            }
            else if tr {
                print!("{:>3}", "T");
            }
            else if map.valid_point(&Point{x, y}){
                print!("{:>3}", ".");
            }
            else {
                print!("{:>3}", "X");
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

    // do the O(n^4) algorithm and then for each pair of points
    // check if the results match
    /*
    fn brute_force() {
        unimplemented!();


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

    pub fn neighbor(&self, p1: &Point, p2: &Point) -> Direction {
        let dist = p1.x.abs_diff(p2.x) + p1.y.abs_diff(p2.y);
        assert!(dist <= 1, "{:?} -> {:?}", p1, p2);
        if dist == 0 {
            return Direction::None;
        }
        else if p1.x < p2.x {
            return Direction::East;
        }
        else if p2.x < p1.x {
            return Direction::West;
        }
        else if p1.y < p2.y {
            return Direction::North;
        }
        else {
            return Direction::South;
        }
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

    */
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
