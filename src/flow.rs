use std::collections::{HashMap, HashSet};

pub trait MaxFlow {
    fn new() -> Self;
    fn reset(&mut self);
    fn add_edge(&mut self, from: i32, to: i32, capacity: i32);
    fn solve(&mut self);
    fn get_flow(&mut self) -> i32;
    fn set_source(&mut self, source: i32);
    fn set_sink(&mut self, sink: i32);
    // return any saturated edge! <if exists>
    fn get_saturated_edge(&self, vertex: i32) -> Option<(i32, i32)>;
    // debug functions
    fn assert_only_one_saturated(&self);
    fn assert_incoming_equals_outgoing(&self);
}

#[derive(Clone)]
pub struct FordFulkerson {
    flow: i32,
    source: i32,
    sink: i32,
    vertices: HashSet<i32>,
    adj: HashMap<i32, Vec<i32>>,
    flows: HashMap<(i32, i32), i32>,
    capacities: HashMap<(i32, i32), i32>,
}

impl FordFulkerson {
    fn dfs(&self, v: i32, vis: &mut HashMap<i32, bool>) -> (i32, Vec<i32>) {
        if v == self.sink { return (std::i32::MAX, vec![v]); }
        vis.insert(v, true);
        //println!("dfs: {}", v);
        //println!("{:?}", vis);
        if !self.adj.contains_key(&v) { return (-1, Vec::new()); }
        for u in self.adj.get(&v).unwrap().iter() {
            if *vis.get(u).unwrap() { continue; }
            let flow_now = self.flows.get(&(v, *u)).unwrap();
            let cap_now = self.capacities.get(&(v, *u)).unwrap();
            //println!("{}->{} {}/{}", v, u, flow_now, cap_now);
            if cap_now-flow_now == 0 { continue; }
            let (mut flow, mut path) = self.dfs(*u, vis);
            //println!("u: {} {}", u, flow);
            if flow == -1 { continue; }
            path.push(v);
            flow = std::cmp::min(flow, cap_now-flow_now);
            return (flow, path);
        }
        return (-1, Vec::new());
    }
}

impl MaxFlow for FordFulkerson {
    fn new() -> Self {
        FordFulkerson {
            flow: -1,
            source: -1,
            sink: -1,
            vertices: HashSet::new(),
            adj: HashMap::new(),
            flows: HashMap::new(),
            capacities: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        self.flow = -1;
        self.source = -1;
        self.sink = -1;
        self.vertices = HashSet::new();
        self.adj = HashMap::new();
        self.flows = HashMap::new();
        self.capacities = HashMap::new();
    }

    fn add_edge(&mut self, from: i32, to: i32, capacity: i32) {
        // println!("from={} to={}", from, to);
        let mut prev = self.adj.remove(&from).unwrap_or(Vec::new());
        prev.push(to);
        self.adj.insert(from, prev);
        let mut prev = self.adj.remove(&to).unwrap_or(Vec::new());
        prev.push(from);
        self.adj.insert(to, prev);

        self.vertices.insert(from);
        self.vertices.insert(to);
        self.flows.insert((from, to), 0);
        self.flows.insert((to, from), 0);
        self.capacities.insert((from, to), capacity);
        self.capacities.insert((to, from), 0);
    }

    fn solve(&mut self) {
        self.flow = 0;
        loop {
            let mut vis = HashMap::new();
            for v in self.vertices.iter() { vis.insert(*v, false); }
            let (flow, path) = self.dfs(self.source, &mut vis);
            if flow == -1 { break; }
            self.flow += flow;
            //println!("{:?}, {}", path, flow);
            for idx in (1..path.len()).rev() {
                let v = path[idx];
                let u = path[idx-1];
                let mut now = self.flows.remove(&(v,u)).unwrap();
                now += flow;
                self.flows.insert((v,u), now);
                now = self.flows.remove(&(u,v)).unwrap();
                now -= flow;
                self.flows.insert((u,v), now);
            }
            //println!("{:?}", self.flows);
        }

        //let smaller = self.flows.iter()
        //    .filter(|e| *e.1 != 0)
        //    .collect::<HashMap<_, _>>();
        //println!("{:?}", smaller);
    }


    fn get_flow(&mut self) -> i32 {
        if self.flow == -1 { self.solve(); }
        //self.assert_only_one_saturated();
        //self.assert_incoming_equals_outgoing();
        self.flow
    }

    fn set_source(&mut self, source: i32) { self.source = source; }
    fn set_sink(&mut self, sink: i32) { self.sink = sink; }

    fn get_saturated_edge(&self, vertex: i32) -> Option<(i32, i32)> {
        if self.adj.contains_key(&vertex) {
            for u in self.adj.get(&vertex).unwrap().iter() {
                let flow = self.flows.get(&(vertex, *u));
                if *flow.unwrap_or(&0) > 0 { return Some((vertex, *u)); }
            }
        }
        None
    }

    fn assert_only_one_saturated(&self) {
        for v in self.vertices.iter() {
            if *v == self.sink || *v == self.source { continue; }
            let mut sum_out = 0;
            let mut sum_in = 0;
            for u in self.adj[v].iter() {
                let val = *self.flows.get(&(*v, *u)).unwrap();
                // skip residuals
                if val > 0 { sum_out += val; }
                let val = *self.flows.get(&(*u, *v)).unwrap();
                // skip residuals
                if val > 0 { sum_in += val; }
            }
            assert!(sum_in <= 1, "v={} in={}", v, sum_in);
        }
    }

    fn assert_incoming_equals_outgoing(&self) {
        for v in self.vertices.iter() {
            if *v == self.sink || *v == self.source { continue; }
            let mut sum_out = 0;
            let mut sum_in = 0;
            for u in self.adj[v].iter() {
                let val = *self.flows.get(&(*v, *u)).unwrap();
                // skip residuals
                if val > 0 { sum_out += val; }
                let val = *self.flows.get(&(*u, *v)).unwrap();
                // skip residuals
                if val > 0 { sum_in += val; }
            }
            assert!(sum_in == sum_out, "in!=out, v={}, in={}, out={}", v, sum_in, sum_out);
        }
    }
}

#[cfg(test)]
mod tests {
   use crate::flow::*;

    #[test]
    fn simple() {
        let mut ff = FordFulkerson::new();
        ff.source = 0;
        ff.sink = 2;
        ff.add_edge(0, 1, 3);
        ff.add_edge(1, 2, 2);
        assert_eq!(ff.vertices, vec![0,1,2].into_iter().collect());

        let mut adj = HashMap::new();
        adj.insert(0, vec![1]);
        adj.insert(1, vec![0, 2]);
        adj.insert(2, vec![1]);
        assert_eq!(ff.adj, adj);

        ff.solve();
        assert_eq!(ff.flow, 2);
    }

    #[test]
    fn classic_4nodes() {
        let mut ff = FordFulkerson::new();
        ff.source = 0;
        ff.sink = 3;
        ff.add_edge(0, 1, 3);
        ff.add_edge(0, 2, 2);
        ff.add_edge(1, 2, 1);
        ff.add_edge(1, 3, 2);
        ff.add_edge(2, 3, 3);
        assert_eq!(ff.vertices, vec![0,1,2,3].into_iter().collect());

        let mut adj = HashMap::new();
        adj.insert(0, vec![1, 2]);
        adj.insert(1, vec![0, 2, 3]);
        adj.insert(2, vec![0, 1, 3]);
        adj.insert(3, vec![1, 2]);
        assert_eq!(ff.adj, adj);

        ff.solve();
        assert_eq!(ff.flow, 5);
    }

    #[test]
    fn ad_exam_network() {
        let mut ff = FordFulkerson::new();
        ff.source = 0;
        ff.sink = 7;
        ff.add_edge(0, 1, 7);
        ff.add_edge(0, 2, 3);
        ff.add_edge(0, 3, 5);

        ff.add_edge(1, 2, 6);
        ff.add_edge(1, 4, 9);

        ff.add_edge(2, 3, 2);
        ff.add_edge(2, 5, 1);

        ff.add_edge(3, 6, 9);

        ff.add_edge(4, 2, 7);
        ff.add_edge(4, 7, 2);

        ff.add_edge(5, 4, 5);
        ff.add_edge(5, 7, 9);

        ff.add_edge(6, 2, 2);
        ff.add_edge(6, 5, 4);
        ff.add_edge(6, 7, 7);

        ff.solve();
        //println!("{:?}", ff.flows);
        //println!("{:?}", ff.capacities);
        assert_eq!(ff.flow, 10);
    }
}
