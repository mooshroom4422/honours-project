use crate::hopcroft_karp::HopcroftKarp;
use crate::map::*;
use crate::matching::{Matcher, makespan_solve};
use crate::flow::MaxFlow;

pub trait AgentStrategy {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub struct MakeSpanHopcroft;
impl AgentStrategy for MakeSpanHopcroft {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = vec![Direction::None; agents.len()];

        let agents_point = agents.into_iter()
            .map(|x| x.position)
            .collect::<Vec<_>>();

        let targets_point = targets.into_iter()
            .map(|x| x.position)
            .collect::<Vec<_>>();

        let mut matcher = HopcroftKarp::new_empty();
        let _dd = makespan_solve(map, &agents_point, &targets_point, &mut matcher);

        let matching = matcher.get_matching();
        for i in 0..agents.len() {
            if matching[i] == -1 { continue; }
            let t = (matching[i] as usize)-agents.len();
            res[i] = map.get_direction(&agents[i].position, &targets[t].position);
        }

        res
    }
}

// free -> free to choose the target to catch
// assigned -> targets are already assigned to each agent
//
// Collision -> collisions are allowed
// NoCollision -> collisions are not allowed
pub struct NoCollisionSingle {
    ready: bool,
    pub goto: Point,
    pub expected_time: i32, // set to -1 if there is no path
}

impl NoCollisionSingle {
    pub fn new() -> Self {
        NoCollisionSingle { ready: false, goto: Point{x:0, y:0}, expected_time: -1}
    }

    pub fn prep(&mut self, map: &Map, agent: &Agent, target: &Target) {
        let mut l = 0;
        let mut r = 1e6 as i32;
        while l <= r {
            let mid = l+(r-l)/2;
            if map.dist_point(&agent.position, &target.at_time(mid as usize)) as i32 <= mid {
                r = mid-1;
                self.goto = target.at_time(mid as usize);
                self.expected_time = mid;
            }
            else {
                l = mid+1;
            }
        }
        self.ready = true;
    }
}

impl AgentStrategy for NoCollisionSingle {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        assert!(agents.len() == 1);
        assert!(self.expected_time != -1);
        if !self.ready { self.prep(map, &agents[0], &targets[0]) }
        vec![map.get_direction(&agents[0].position, &self.goto)]
    }
}

pub struct CollisionAssigned {
    ready: bool,
    goto: Vec<Point>,
}

impl CollisionAssigned {
    pub fn new() -> Self {
        CollisionAssigned { ready: false, goto: Vec::new() }
    }

    pub fn prep(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>, permutation: &Vec<usize>) {
        assert!(agents.len() == targets.len());
        assert!(agents.len() == permutation.len());
        self.goto = vec![Point{x:0, y:0}; agents.len()];
        for (idx, agent) in agents.iter().enumerate() {
            let mut single_strat = NoCollisionSingle::new();
            single_strat.prep(map, agent, &targets[permutation[idx]]);
            assert!(single_strat.expected_time != -1);
            self.goto[idx] = single_strat.goto;
            //println!("{}: {:?}", idx, single_strat.expected_time);
        }
        self.ready = true;
    }
}

impl AgentStrategy for CollisionAssigned {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        assert!(self.ready);
        let mut res = vec![Direction::None; agents.len()];
        for (idx, agent) in agents.iter().enumerate() {
            res[idx] = map.get_direction(&agent.position, &self.goto[idx]);
        }
        res
    }
}

pub struct CollisionFree {
    ready: bool,
    goto: Vec<Point>,
}

impl CollisionFree {
    pub fn new() -> Self {
        CollisionFree { ready: false, goto: Vec::new() }
    }

    pub fn prep(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>, matcher: &mut impl Matcher) {
        assert!(agents.len() == targets.len());

        // find permuatation
        let mut left: i32 = 0;
        let mut right: i32 = 1_000_000_000;
        let n = agents.len();
        let m = targets.len();
        let mut perm = vec![0; n];

        while left <= right {
            let mid = left+(right-left)/2;
            // println!("trying... {}", mid);
            let mut graph: Vec<Vec<usize>> = vec![Vec::new(); n+m];
            for (i, agent) in agents.iter().enumerate() {
                for (j, target) in targets.iter().enumerate() {
                    let mut single_strat = NoCollisionSingle::new();
                    single_strat.prep(map, agent, target);
                    if single_strat.expected_time == -1 { continue; }
                    if single_strat.expected_time <= mid {
                        graph[i].push(j+n);
                        graph[j+n].push(i);
                    }
                }
            }
            let setu = (0..agents.len()).collect();
            let setv = (agents.len()..agents.len()+targets.len()).collect();
            matcher.init(graph, setu, setv);
            let got = matcher.solve();
            // println!("got: {}", got);
            if got == n { // assuming n == m
                right = mid-1;
                for idx in 0..n {
                    perm[idx] = (matcher.get_matching()[idx]-(n as i32)) as usize;
                }
            }
            else {
                left = mid+1;
            }
        }

        //println!("permutation: {:?}", perm);

        // get the result from CollisionAssigned using found permutation
        let mut assigned = CollisionAssigned::new();
        assigned.prep(map, agents, targets, &perm);
        self.goto = assigned.goto;
        self.ready = true;
    }
}

impl AgentStrategy for CollisionFree {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        assert!(self.ready);
        let mut res = vec![Direction::None; agents.len()];
        for (idx, agent) in agents.iter().enumerate() {
            res[idx] = map.get_direction(&agent.position, &self.goto[idx]);
        }
        res
    }
}

pub struct NoCollisionFree {
    height: i32,
    width: i32,
    ready: bool,
    paths_idx: Vec<usize>,
    paths: Vec<Vec<Direction>>,
}

impl NoCollisionFree {
    pub fn new() -> Self {
        NoCollisionFree {
            height: -1,
            width: -1,
            ready: false,
            paths_idx: Vec::new(),
            paths: Vec::new(),
        }
    }

    fn conv_expl(&self, time: i32, x: i32, y: i32) -> i32 {
        return time*self.width*self.height+x*self.width+y;
    }

    fn conv(&self, time: i32, pnt: &Point) -> i32 {
        return self.conv_expl(time, pnt.x as i32, pnt.y as i32);
    }

    fn reconv_point(&self, time: i32, pos: i32) -> Point {
        let mut notime = pos;
        if time > 0 { notime = pos%(time*self.width*self.height); }
        let x = (notime/self.width) as usize;
        let y = (notime%self.width) as usize;
        Point {x, y}
    }

    fn construct(&self, flow: &mut impl MaxFlow, sink: i32, source: i32, mid: i32,
                 map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) {

        let directions = vec![Direction::North, Direction::East,
                              Direction::South, Direction::West,
                              Direction::None];
        flow.reset();
        flow.set_source(source);
        flow.set_sink(sink);
        for agent in agents.iter() {
            flow.add_edge(source, self.conv(0, &agent.position), 1);
        }
        for target in targets.iter() {
            flow.add_edge(self.conv(0, &target.position), sink, 1);
        }

        for timer in 0..mid {
            for x in 0..self.height {
                for y in 0..self.width {
                    let pnt = Point{x: x as usize, y: y as usize};
                    for dir in directions.iter() {
                        if !map.valid_direction(pnt, *dir) { continue; }
                        let nxt = go_direction(pnt, *dir);
                        if !map.valid_point(nxt) { continue; }
                        flow.add_edge(self.conv(2*timer, &pnt), self.conv(2*timer+1, &nxt), 1);
                    }
                    // filter
                    flow.add_edge(self.conv(2*timer+1, &pnt), self.conv(2*timer+2, &pnt), 1);
                }
            }
        }

        for target in targets.iter() {
            //println!("{:?} {} {:?}", &target.at_time(mid as usize),
            //         self.conv(2*mid, &target.at_time(mid as usize)),
            //         self.reconv_point(2*mid, self.conv(2*mid, &target.at_time(mid as usize))));
            flow.add_edge(self.conv(2*mid, &target.at_time(mid as usize)), sink, 1);
        }

    }

    pub fn prep(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>, flow: &mut impl MaxFlow) {
        self.height = map.height as i32;
        self.width = map.width as i32;

        let source = -1;
        let sink = -2;

        let mut left: i32 = 0;
        let mut right: i32 = 100;
        let mut res: i32 = -1;

        while left <= right {
            let mid = left+(right-left)/2;

            self.construct(flow, sink, source, mid, map, agents, targets);
            println!("finished construction");

            if flow.get_flow() == agents.len() as i32 {
                right = mid-1;
                res = mid;
            }
            else {
                left = mid+1;
            }

            println!("finished: {}, {}, l:{}, r:{}", mid, res, left, right);
        }

        println!("res: {}", res);
        if res == -1 {
            println!("couldnt find any path :(");
            panic!();
        }

        self.paths_idx = vec![0; agents.len()];
        self.paths = vec![Vec::new(); agents.len()];

        self.construct(flow, sink, source, res, map, agents, targets);
        flow.solve();

        let mut last_pos = vec![Point{x:0, y:0}; agents.len()];
        for (idx, agent) in agents.iter().enumerate() {
            last_pos[idx] = agent.position;
        }

        for timer in 0..res {
            for idx in 0..agents.len() {
                let conv_pos = self.conv(2*timer, &last_pos[idx]);
                let edge = flow.get_saturated_edge(conv_pos);
                if edge.is_none() {
                    //println!("dead end: t={} idx={} x={} y={}", timer, idx, last_pos[idx].x, last_pos[idx].y);
                }
                //print!("added: t={} idx={} x={} y={}", timer, idx, last_pos[idx].x, last_pos[idx].y);
                let npos = edge.unwrap().1;
                let npoint = self.reconv_point(2*timer+1, npos);
                self.paths[idx].push(map.get_direction(&last_pos[idx], &npoint));
                last_pos[idx] = npoint;
                //print!(" -> x={} y={}\n", last_pos[idx].x, last_pos[idx].y);
            }
        }

        self.ready = true;
    }
}

impl AgentStrategy for NoCollisionFree {
    fn pick(&mut self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = Vec::new();
        for idx in 0..agents.len() {
            if self.paths_idx[idx] >= self.paths[idx].len() {
                res.push(Direction::None);
                continue;
            }
            res.push(self.paths[idx][self.paths_idx[idx]]);
            self.paths_idx[idx] += 1;
        }

        res
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conv() {
        let mut strat = NoCollisionFree::new();
        strat.height = 10;
        strat.width = 10;

        let got = strat.conv_expl(0, 3, 5);
        let reconv = strat.reconv_point(0, got);
        assert_eq!(Point{x:3, y:5}, reconv);

        let got = strat.conv_expl(4, 3, 5);
        let reconv = strat.reconv_point(4, got);
        assert_eq!(Point{x:3, y:5}, reconv);

        let pnt = Point{x:7, y:3};
        let got = strat.conv(3, &pnt);
        let reconv = strat.reconv_point(3, got);
        assert_eq!(pnt, reconv);
    }
}
