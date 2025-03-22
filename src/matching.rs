use super::map::*;

pub trait Matcher {
    fn new_from_graph(graph: Vec<Vec<usize>>, setu: Vec<usize>, setv: Vec<usize>) -> Self;
    fn new() -> Self;
    fn init(&mut self, graph: Vec<Vec<usize>>, setu: Vec<usize>, setv: Vec<usize>);

    // returns matching size
    fn solve(&mut self) -> usize;

    // returns vector with selected vertices
    fn get_matching(&mut self) -> &Vec<i32>;
}

pub fn makespan_solve(map: &Map, agents: &Vec<Point>, targets: &Vec<Point>, matcher: &mut impl Matcher) -> i32 {
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
                if map.dist_point(&agent, &target) <= mid as usize {
                    graph[i].push(j+n);
                    graph[j+n].push(i);
                }
            }
        }

        let setu = (0..agents.len()).collect();
        let setv = (agents.len()..agents.len()+targets.len()).collect();
        // println!("graph: {:?}", graph);
        // println!("setu: {:?}", setu);
        // println!("setv: {:?}", setv);
        // println!("matching: {:?}", matcher.get_matching());
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

    // bringback matcher to state with optimal mid
    if res != -1 {
        let mut graph: Vec<Vec<usize>> = vec![Vec::new(); n+m];
        for (i, agent) in agents.iter().enumerate() {
            for (j, target) in targets.iter().enumerate() {
                if map.dist_point(&agent, &target) <= res as usize {
                    graph[i].push(j+n);
                    graph[j+n].push(i);
                }
            }
        }

        let setu = (0..agents.len()).collect();
        let setv = (agents.len()..agents.len()+targets.len()).collect();
        matcher.init(graph, setu, setv);
        _ = matcher.solve();
    }

    res
}

#[cfg(test)]
mod tests {
    use crate::hopcroft_karp::HopcroftKarp;
    use crate::turbo::TurboMatching;
    use crate::map::*;

    use super::*;

    #[test]
    fn simple() {
        let agents = vec![Point{ x: 1, y: 1 }];
        let targets = vec![Point{ x: 3, y: 3 }];
        let mut matcher = TurboMatching{ g: Vec::new(), vis: Vec::new(), mat: Vec::new(),};
        let map = Map::new("resources/maps/example.map");
        assert_eq!(makespan_solve(&map, &agents, &targets, &mut matcher), 4);
    }

    #[test]
    fn two_agents() {
        let agents = vec![Point{ x: 2, y: 1 }, Point{ x: 3, y: 1 }];
        let targets = vec![Point{ x: 1, y: 2 }, Point{ x: 2, y: 3 }];
        let mut matcher = HopcroftKarp {
            g: Vec::new(),
            setu: Vec::new(),
            setv: Vec::new(),
            pairu: Vec::new(),
            pairv: Vec::new(),
            dist: Vec::new(),
            matching: Vec::new(),
            NIL: 0,
        };
        let map = Map::new("resources/maps/example.map");
        assert_eq!(makespan_solve(&map, &agents, &targets, &mut matcher), 3);
    }
}
