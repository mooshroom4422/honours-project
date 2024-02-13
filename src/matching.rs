struct Point {
    x: i32,
    y: i32,
}

pub trait Matcher {
    fn new(graph: Vec<Vec<usize>>, setu: Vec<usize>, setv: Vec<usize>) -> Self;
    fn init(&mut self, graph: Vec<Vec<usize>>, setu: Vec<usize>, setv: Vec<usize>);

    // returns matching size
    fn solve(&mut self) -> usize;

    // returns vector with selected vertices
    fn get_matching(&mut self) -> &Vec<i32>;
}

// temporary solution, ignores walls
#[inline(always)]
fn dist(u: &Point, v: &Point) -> i32 {
    return i32::abs(u.x-v.x) + i32::abs(u.y-v.y);
}

fn solve(agents: &Vec<Point>, targets: &Vec<Point>, matcher: &mut impl Matcher) -> i32 {
    let mut left: i32 = 0;
    let mut right: i32 = 1_000_000_000;

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

        let setu = (0..agents.len()).collect();
        let setv = (agents.len()..agents.len()+targets.len()).collect();
        matcher.init(graph, setu, setv);
        let got = matcher.solve();
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

    use crate::turbo::TurboMatching;

    use super::{solve, Point};

    #[test]
    fn simple() {
        let agents = vec![Point{ x: 0, y: 0 }];
        let targets = vec![Point{ x: 2, y: 2 }];
        let mut matcher = TurboMatching{ g: Vec::new(), mat: Vec::new(), vis: Vec::new(), };
        assert_eq!(solve(&agents, &targets, &mut matcher), 4);
    }

    #[test]
    fn three_agents() {
        let agents = vec![Point{ x: 0, y: 0 }, Point{ x: 0, y: 10 }, Point{ x: -10, y: 40}]; let targets = vec![Point{ x: 5, y: 0 }, Point{ x: 5, y: 10 }, Point{ x: 2, y: 38}];
        let mut matcher = TurboMatching{ g: Vec::new(), mat: Vec::new(), vis: Vec::new(), };
        assert_eq!(solve(&agents, &targets, &mut matcher), 14);
    }

    #[test]
    fn many_points() {
        let mut agents: Vec<Point> = Vec::new();
        let mut targets: Vec<Point> = Vec::new();

        let mut rng = rand::thread_rng();
        let mil = 1_000_000;
        for _ in 0..2000 {
            agents.push(Point{ x: rng.gen_range(-mil..mil), y: rng.gen_range(-mil..mil) });
            targets.push(Point{ x: rng.gen_range(-mil..mil), y: rng.gen_range(-mil..mil) });
        }

        let mut matcher = TurboMatching{ g: Vec::new(), mat: Vec::new(), vis: Vec::new(), };
        let got = solve(&agents, &targets, &mut matcher);
        println!("{}", got);
    }
}
