use std::{fs, collections::VecDeque};

#[derive(Clone, Debug, PartialEq)]
enum Tile {
    Wall,
    Free,
}

struct Map {
    height: usize,
    width: usize,
    map: Vec<Vec<Tile>>,
    dist: Vec<Vec<usize>>,
}

impl Map {
    fn new(file_path: &str) -> Self {
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
        };

        fn go(stac: usize, now: usize, x: usize, y:usize, q: &mut VecDeque<(usize, usize)>,
              dist: &mut Vec<Vec<usize>>, res: &mut Map) {
            let to = res.conv(x, y);
            if dist[stac][to] == usize::MAX {
                q.push_back((x, y));
                dist[stac][to] = dist[stac][now]+1;
            }
        }

        let tiles = height*width;
        let mut dist = vec![vec![usize::MAX; tiles]; tiles];
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
                        go(stac, now, x-1, y, &mut q, &mut dist, &mut res);
                    }

                    if x < height-1 && res.valid(x+1, y) {
                        go(stac, now, x+1, y, &mut q, &mut dist, &mut res);
                    }

                    if y > 0 && res.valid(x, y-1) {
                        go(stac, now, x, y-1, &mut q, &mut dist, &mut res);
                    }

                    if y < width-1 && res.valid(x, y+1) {
                        go(stac, now, x, y+1, &mut q, &mut dist, &mut res);
                    }
               }
            }
        }

        res.dist = dist;

        res
    }

    fn conv(&self, x: usize, y: usize) -> usize {
        return x*self.width+y;
    }

    fn valid(&self, x: usize, y: usize) -> bool {
        self.map[x][y] == Tile::Free
    }

    fn dist(&self, fx: usize, fy: usize, tx: usize, ty: usize) -> usize {
        if !self.valid(fx, fy) || !self.valid(tx, ty) {
            return usize::MAX;
        }
        return self.dist[self.conv(fx, fy)][self.conv(tx, ty)];
    }

    fn get_dist(&self, x:usize, y: usize) -> Vec<usize> {
        self.dist[self.conv(x, y)].clone()
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
        assert_eq!(exp, map.get_dist(0, 0));

        exp = Vec::from([
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, 0, 1, 2, usize::MAX,
                usize::MAX, 1, usize::MAX, 3, usize::MAX,
                usize::MAX, 2, 3, 4, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
            ]);
        assert_eq!(exp, map.get_dist(1, 1));

        exp = Vec::from([
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
                usize::MAX, 3, 4, 3, usize::MAX,
                usize::MAX, 2, usize::MAX, 2, usize::MAX,
                usize::MAX, 1, 0, 1, usize::MAX,
                usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX,
            ]);
        assert_eq!(exp, map.get_dist(3, 2));

    }
}
