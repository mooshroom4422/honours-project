use crate::hopcroft_karp::HopcroftKarp;
use crate::map::*;
use crate::matching::{Matcher, makespan_solve};
use crate::flow::MaxFlow;
use log::info;
use std::time::Instant;

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub enum AgentStrategies {
    MakeSpanHopcroft,
    NoCollisionSingle,
    CollisionAssigned,
    CollisionFree,
    NoCollisionFree,
}

pub trait AgentStrategy {
    fn pick(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub struct MakeSpanHopcroft;
impl AgentStrategy for MakeSpanHopcroft {
    fn pick(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        let mut res = vec![Direction::None; agents.len()];

        let agents_points = agents.into_iter()
            .filter(|x| x.active)
            .map(|x| x.position)
            .collect::<Vec<_>>();

        let idxs = agents.into_iter()
            .enumerate()
            .filter(|x| x.1.active)
            .map(|x| x.0)
            .collect::<Vec<_>>();

        let targets_points = targets.into_iter()
            .map(|x| x.position)
            .collect::<Vec<_>>();

        let mut matcher = HopcroftKarp::new();
        let _dd = makespan_solve(map, &agents_points, &targets_points, &mut matcher);

        let matching = matcher.get_matching();
        for i in 0..agents_points.len() {
            if matching[i] == -1 { continue; }
            let t = (matching[i] as usize)-agents_points.len();
            res[idxs[i]] = map.get_direction(&agents[idxs[i]].position, &targets[t].position);
            agents[idxs[i]].targets = targets[t].idx as i32;
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
        let mut r = 2048 as i32;
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
    fn pick(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
        assert!(agents.len() == 1);
        assert!(self.expected_time != -1);
        if !self.ready { self.prep(map, &agents[0], &targets[0]) }
        vec![map.get_direction(&agents[0].position, &self.goto)]
    }
}

#[derive(Clone)]
pub struct CollisionAssigned {
    ready: bool,
    goto: Vec<Point>,
}

impl CollisionAssigned {
    pub fn new() -> Self {
        CollisionAssigned { ready: false, goto: Vec::new() }
    }

    pub fn prep(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>, permutation: &Vec<usize>) {
        assert!(agents.len() == targets.len());
        assert!(agents.len() == permutation.len());
        self.goto = vec![Point{x:0, y:0}; agents.len()];
        for (idx, agent) in agents.iter_mut().enumerate() {
            let mut single_strat = NoCollisionSingle::new();
            single_strat.prep(map, agent, &targets[permutation[idx]]);
            assert!(single_strat.expected_time != -1);
            self.goto[idx] = single_strat.goto;
            agent.targets = permutation[idx] as i32;
            // println!("{}: {:?}", idx, single_strat.expected_time);
        }
        self.ready = true;
    }
}

impl AgentStrategy for CollisionAssigned {
    fn pick(&mut self, map: &Map, agents: &mut Vec<Agent>, _targets: &Vec<Target>) -> Vec<Direction> {
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

    pub fn prep(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>, matcher: &mut impl Matcher) {
        assert!(agents.len() == targets.len());

        let start = Instant::now();
        // find permuatation
        let mut left: i32 = 0;
        // has to be more than the maximum expected search length
        let mut right: i32 = 2048;
        let n = agents.len();
        let m = targets.len();
        let mut perm = vec![0; n];

        let INF = 1e9 as i32 + 999;
        // preproccess the distances to save some oracle time
        let mut preproc = vec![vec![INF; targets.len()]; agents.len()];
        for (i, agent) in agents.iter().enumerate() {
            for (j, target) in targets.iter().enumerate() {
                let mut single_strat = NoCollisionSingle::new();
                single_strat.prep(map, agent, target);
                preproc[i][j] = single_strat.expected_time;
            }
        }

        while left <= right {
            let mid = left+(right-left)/2;
            // println!("trying... {}", mid);
            let mut graph: Vec<Vec<usize>> = vec![Vec::new(); n+m];
            for (i, agent) in agents.iter().enumerate() {
                for (j, target) in targets.iter().enumerate() {
                    // let mut single_strat = NoCollisionSingle::new();
                    // single_strat.prep(map, agent, target);
                    if preproc[i][j] == -1 { continue; }
                    if preproc[i][j] <= mid {
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

        // println!("permutation: {:?}", perm);

        // get the result from CollisionAssigned using found permutation
        let mut assigned = CollisionAssigned::new();
        assigned.prep(map, agents, targets, &perm);
        self.goto = assigned.goto;
        self.ready = true;

        // info!("prep took: {:?}", start.elapsed());
    }
}

impl AgentStrategy for CollisionFree {
    fn pick(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
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
    // grid and two filters
    const LAYERS: i32 = 3;
    // number of edges that we map on egdes filter
    const DIRECTIONS: i32 = 3;

    pub fn new() -> Self {
        NoCollisionFree {
            height: -1,
            width: -1,
            ready: false,
            paths_idx: Vec::new(),
            paths: Vec::new(),
        }
    }

    fn conv_expl(&self, layer: i32, x: i32, y: i32) -> i32 {
        return layer*self.width*self.height*Self::DIRECTIONS+(y*self.width+x);
    }

    fn conv(&self, layer: i32, pnt: &Point) -> i32 {
        return self.conv_expl(layer, pnt.x as i32, pnt.y as i32);
    }

    fn conv_edge_expl(&self, layer: i32, x: i32, y: i32, dir: &Direction) -> i32 {
        let cell = self.conv_expl(layer, x*3, y*3);
        // no move -> special 3rd, or just skip filter
        // e, s -> normal
        // w, n -> move back and create edge
        // num edges -> 3n
        if *dir == Direction::None { return cell; }
        else if *dir == Direction::East || *dir == Direction::South {
            let mut mv = 1;
            if *dir == Direction::South { mv = 2; }
            // println!("conv_edge: {} {} {:?} -> {} {}", x, y, dir, cell, mv);
            return cell + mv;
        }
        else {
            let mut tmp = Point { x: x as usize, y: y as usize };
            tmp = go_direction(tmp, *dir);
            // println!("conv_edge: {} {} {:?} -> {} {} {:?}", x, y, dir, tmp.x, tmp.y, Map::reverse_direction(dir));
            return self.conv_edge(layer, &tmp, &Map::reverse_direction(dir));
        }
    }

    fn conv_edge(&self, layer: i32, pnt: &Point, dir: &Direction) -> i32 {
        return self.conv_edge_expl(layer, pnt.x as i32, pnt.y as i32, dir);
    }

    fn reconv_point(&self, time: i32, pos: i32) -> Point {
        let mut notime = pos;
        if time > 0 { notime = pos%(time*self.width*self.height*Self::DIRECTIONS); }
        let x = (notime%self.width) as usize;
        let y = (notime/self.width) as usize;
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
        // println!("source={}, sink={}", source, sink);
        // println!("connecting source to agents:");
        for agent in agents.iter() {
            flow.add_edge(source, self.conv(0, &agent.position), 1);
        }
        // println!("======");

        for timer in 0..mid {
            for x in 0..self.width {
                for y in 0..self.height {
                    let pnt = Point{x: x as usize, y: y as usize};
                    for dir in directions.iter() {
                        if !map.valid_point(&pnt) || !map.valid_direction(pnt, *dir) { continue; }
                        let nxt = go_direction(pnt, *dir);
                        if !map.valid_point(&nxt) { continue; }
                        // println!("trying: pnt={:?} {:?} -> nxt={:?}", pnt, *dir, nxt);
                        let conv_edge = self.conv_edge(Self::LAYERS*timer+1, &pnt, dir);
                        flow.add_edge(self.conv(Self::LAYERS*timer, &pnt), conv_edge, 1);
                        flow.add_edge(conv_edge, self.conv(Self::LAYERS*timer+2, &nxt), 1); // zle double edges
                    }
                    // cell filter
                    // println!("cell filter: timer={} x={} y={}", timer, x, y);
                    flow.add_edge(self.conv(Self::LAYERS*timer+2, &pnt), self.conv(Self::LAYERS*timer+3, &pnt), 1);
                }
            }
        }

        let mut collector = sink-1; // assuming sink is the smallest node (in terms of id)
        for target in targets.iter() {
            // println!("{:?} {} {:?}", &target.at_time(mid as usize),
            //         self.conv(2*mid, &target.at_time(mid as usize)),
            //         self.reconv_point(2*mid, self.conv(2*mid, &target.at_time(mid as usize))));
            flow.add_edge(collector, sink, 1);
            for time in 0..mid+1 {
                flow.add_edge(self.conv(Self::LAYERS*time, &target.at_time(time as usize)), collector, 1);
            }
            collector -= 1;
        }

    }

    pub fn prep(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>, flow: &mut impl MaxFlow) {
        self.height = map.height as i32;
        self.width = map.width as i32;

        let source = -1;
        let sink = -2;

        let mut left: i32 = 0;
        let mut right: i32 = 150;
        let mut res: i32 = -1;

        while left <= right {
            let mid = left+(right-left)/2;
            // println!("mid={}", mid);
            // println!("agents={:?}", agents);
            // println!("targets={:?}", targets);

            self.construct(flow, sink, source, mid, map, agents, targets);
            // println!("finished construction");

            if flow.get_flow() == agents.len() as i32 {
                right = mid-1;
                res = mid;
            }
            else {
                left = mid+1;
            }

            // println!("finished: {}, {}, l:{}, r:{}", mid, res, left, right);
            // println!("flow={}", flow.get_flow());
            // panic!();
        }

        // println!("res: {}", res);
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

        // println!("res: {}", res);
        for timer in 0..res {
            for idx in 0..agents.len() {
                let conv_pos = self.conv(Self::LAYERS*timer, &last_pos[idx]);
                let mut edge = flow.get_saturated_edge(conv_pos);
                // println!("\nconv_pos={}", conv_pos);
                if edge.is_none() {
                    // println!("dead end: t={} idx={} x={} y={}", timer, idx, last_pos[idx].x, last_pos[idx].y);
                    panic!("failed to recontruct path");
                }
                // println!("added: t={} idx={} x={} y={}", timer, idx, last_pos[idx].x, last_pos[idx].y);
                let edge_filter = edge.unwrap().1;
                // println!("edge_filter: {}", edge_filter);
                let mut edge = flow.get_saturated_edge(edge_filter);
                let cell_filter = edge.unwrap().1;
                // println!("cell_filter: {}", cell_filter);
                let mut edge = flow.get_saturated_edge(cell_filter);
                let next_cell = edge.unwrap().1;
                // println!("next_cell: {}", next_cell);
                let npoint = self.reconv_point(Self::LAYERS*(timer+1), next_cell);
                self.paths[idx].push(map.neighbor(&last_pos[idx], &npoint));
                last_pos[idx] = npoint;
                // println!(" -> x={} y={} dir={:?}\n", last_pos[idx].x, last_pos[idx].y, self.paths[idx].last());
            }
        }

        for idx in 0..agents.len() {
            let conv_pos = self.conv(Self::LAYERS*res, &last_pos[idx]);
            let edge = flow.get_saturated_edge(conv_pos);
            // println!("next: {}", edge.unwrap().1);
            let target_id = -(edge.unwrap().1 - (-3));
            agents[idx].targets = target_id;
        }

        self.ready = true;
    }
}

impl AgentStrategy for NoCollisionFree {
    fn pick(&mut self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
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
