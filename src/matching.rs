struct Point {
    x: i32,
    y: i32,
}

fn dfs(v: usize, g: &Vec<Vec<usize>>, vis: &mut Vec<bool>, mat: &mut Vec<i32>) -> bool {
    vis[v] = true;
    for &u in g[v].iter() {
        if mat[u] == -1 || (!vis[mat[u] as usize] && dfs(mat[u] as usize, g, vis, mat)) {
            mat[u] = v as i32;
            mat[v] = u as i32;
            return true;
        }
    }
    return false;
}

// dfs matching with heuristic, hopcroft-karp was too hard to implement rn :(
// returns size of matching, leaves matching in mat
// average performance O(n)
// worst case O(n*m)
fn matching(g: &Vec<Vec<usize>>, mat: &mut Vec<i32>) -> usize {
    let n = g.len();
    let mut changed = true;
    let mut result = 0;
    let mut vis = vec![false; n];
    while changed {
        changed = false;
        vis.fill(false);
        for v in 0..n {
            if mat[v] == -1 && dfs(v, &g, &mut vis, mat) {
                changed = true;
                result += 1;
            }
        }
    }
    return result;
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

        let mut mat = vec![-1; n+m];
        let got = matching(&graph, &mut mat);
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
