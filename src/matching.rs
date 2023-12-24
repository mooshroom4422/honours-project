struct Point {
    x: i32,
    y: i32,
}

pub trait Matcher {
    fn new(graph: Vec<Vec<usize>>) -> Self;

    // returns matching size
    fn solve(&mut self) -> usize;

    // returns hashmap with selected vertices
    fn get_matching(&self) -> &Vec<i32>;
}

pub struct TurboMatching {
    g: Vec<Vec<usize>>,
    vis: Vec<bool>,
    mat: Vec<i32>,
}

impl TurboMatching {
    fn dfs(&mut self, v: usize) -> bool {
        self.vis[v] = true;
        for idx in 0..self.g[v].len() {
            let u = self.g[v][idx];
            if self.mat[u] == -1 || (!self.vis[self.mat[u] as usize] && self.dfs(self.mat[u] as usize)) {
                self.mat[u] = v as i32;
                self.mat[v] = u as i32;
                return true;
            }
        }
        return false;
    }

    // dfs matching with heuristic
    // average performance O(n)
    // worst case O(n*m)
    fn matching(&mut self) -> usize {
        let n = self.g.len();
        let mut changed = true;
        let mut result = 0;
        let mut vis = vec![false; n];
        while changed {
            changed = false;
            vis.fill(false);
            for v in 0..n {
                if self.mat[v] == -1 && self.dfs(v) {
                    changed = true;
                    result += 1;
                }
            }
        }
        result
    }
}

impl Matcher for TurboMatching {
    fn new(graph: Vec<Vec<usize>>) -> Self {
        let n: usize = graph.len();
        TurboMatching {
            g: graph,
            vis: vec![false; n],
            mat: vec![-1; n],
        }
    }

    fn solve(&mut self) -> usize {
        self.matching()
    }

    fn get_matching(&self) -> &Vec<i32> {
        &self.mat
    }
}

// temporary solution, ignores walls
#[inline(always)]
fn dist(u: &Point, v: &Point) -> i32 {
    return i32::abs(u.x-v.x) + i32::abs(u.y-v.y);
}

fn solve(agents: &Vec<Point>, targets: &Vec<Point>) -> i32 {
    let mut left: i32 = 0;
    let mut right: i32 = 1000*1000*1000;

    let n = agents.len();
    let m = targets.len();
    let mut res = -1;

    while left <= right {
        let mid = left+(right-left)/2;
        // println!("trying... {}", mid);
        let mut graph: Vec<Vec<usize>> = vec![Vec::new(); n+m];
        for (i, agent) in agents.iter().enumerate() {
            for (j, target) in targets.iter().enumerate() {
                if dist(&agent, &target) <= mid {
                    graph[i].push(j+n);
                    graph[j+n].push(i);
                }
            }
        }

        let mut mat = TurboMatching::new(graph);
        let got = mat.solve();
        if got == std::cmp::min(n, m) {
            res = mid;
            right = mid-1;
        }
        else {
            left = mid+1;
        }
    }

    return res;
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{solve, Point};

    #[test]
    fn simple() {
        let agents = vec![Point{ x: 0, y: 0 }];
        let targets = vec![Point{ x: 2, y: 2 }];
        assert_eq!(solve(&agents, &targets), 4);
    }

    #[test]
    fn three_agents() {
        let agents = vec![Point{ x: 0, y: 0 }, Point{ x: 0, y: 10 }, Point{ x: -10, y: 40}];
        let targets = vec![Point{ x: 5, y: 0 }, Point{ x: 5, y: 10 }, Point{ x: 2, y: 38}];
        assert_eq!(solve(&agents, &targets), 14);
    }

    #[test]
    fn many_points() {
        let mut agents: Vec<Point> = Vec::new();
        let mut targets: Vec<Point> = Vec::new();

        let mut rng = rand::thread_rng();
        let mil = 1000*1000;
        for _ in 0..2000 {
            agents.push(Point{ x: rng.gen_range(-mil..mil), y: rng.gen_range(-mil..mil) });
            targets.push(Point{ x: rng.gen_range(-mil..mil), y: rng.gen_range(-mil..mil) });
        }

        let got = solve(&agents, &targets);
        println!("{}", got);
    }
}
