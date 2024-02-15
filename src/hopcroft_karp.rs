use std::collections::VecDeque;

use crate::matching::Matcher;

const INF: usize = usize::MAX;

// https://en.wikipedia.org/wiki/Hopcroft%E2%80%93Karp_algorithm
#[allow(non_snake_case)]
pub struct HopcroftKarp {
    pub g: Vec<Vec<usize>>,
    pub setu: Vec<usize>,
    pub setv: Vec<usize>,
    pub pairu: Vec<usize>,
    pub pairv: Vec<usize>,
    pub dist: Vec<usize>,
    pub matching: Vec<i32>,
    pub NIL: usize,
}

impl HopcroftKarp {
    fn bfs(&mut self) -> bool {
        let mut q: VecDeque<usize> = VecDeque::new();
        for &u in self.setu.iter() {
            if self.pairu[u] == self.NIL {
                self.dist[u] = 0;
                q.push_back(u);
            }
            else {
                self.dist[u] = INF;
            }
        }
        self.dist[self.NIL] = INF;
        while !q.is_empty() {
            let u = q.pop_front().unwrap();
            if self.dist[u] < self.dist[self.NIL] {
                for &v in self.g[u].iter() {
                    if self.dist[self.pairv[v]] == INF {
                        self.dist[self.pairv[v]] = self.dist[u]+1;
                        q.push_back(self.pairv[v]);
                    }
                }
            }
        }
        self.dist[self.NIL] != INF
    }

    fn dfs(&mut self, u: usize) -> bool {
        if u != self.NIL {
            for idx in 0..self.g[u].len() {
                let v = self.g[u][idx];
                if self.dist[self.pairv[v]] == self.dist[u]+1 {
                    if self.dfs(self.pairv[v]) {
                        self.pairv[v] = u;
                        self.pairu[u] = v;
                        return true;
                    }
                }
            }
            self.dist[u] = INF;
            return false;
        }
        return true;
    }
}

impl Matcher for HopcroftKarp {
    fn new(graph: Vec<Vec<usize>>, setu_in: Vec<usize>, setv_in: Vec<usize>) -> Self {
        let nil = graph.len();
        let n = graph.len()+1;
        HopcroftKarp {
            g: graph,
            setu: setu_in,
            setv: setv_in,
            pairu: vec![nil; n],
            pairv: vec![nil; n],
            dist: vec![INF; n],
            matching: vec![-1; n],
            NIL: nil,
        }
    }

    fn new_empty() -> Self {
        HopcroftKarp {
            g: Vec::new(),
            setu: Vec::new(),
            setv: Vec::new(),
            pairu: Vec::new(),
            pairv: Vec::new(),
            dist: Vec::new(),
            matching: Vec::new(),
            NIL: 0,
        }
    }

    fn init(&mut self, graph: Vec<Vec<usize>>, setu_in: Vec<usize>, setv_in: Vec<usize>) {
        let nil = graph.len();
        let n = graph.len()+1;
        self.g = graph;
        self.setu = setu_in;
        self.setv = setv_in;
        self.pairu = vec![nil; n];
        self.pairv = vec![nil; n];
        self.dist = vec![INF; n];
        self.matching = vec![-1; n];
        self.NIL = nil;
    }

    fn solve(&mut self) -> usize {
        for &u in self.setu.iter() {
            self.pairu[u] = self.NIL;
        }
        for &v in self.setv.iter() {
            self.pairv[v] = self.NIL;
        }
        let mut matching = 0;
        while self.bfs() {
            for idx in 0..self.setu.len() {
                let u = self.setu[idx];
                if self.pairu[u] == self.NIL && self.dfs(u) {
                    matching += 1;
                }
            }
        }
        matching
    }

    fn get_matching(&mut self) -> &Vec<i32> {
        for &u in self.setu.iter() {
            if self.pairu[u] != self.NIL {
                self.matching[u] = self.pairu[u] as i32;
            }
            else {
                self.matching[u] = -1;
            }
        }
        for &v in self.setv.iter() {
            if self.pairv[v] != self.NIL {
                self.matching[v] = self.pairv[v] as i32;
            }
            else {
                self.matching[v] = -1;
            }
        }
        &self.matching
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use ntest::timeout;
    use crate::hopcroft_karp::HopcroftKarp;
    use crate::matching::Matcher;

    fn add_edge(g: &mut Vec<Vec<usize>>, v: usize, u: usize) {
        g[v].push(u);
        g[u].push(v);
    }

    #[test]
    fn simple() {
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 4];
        add_edge(&mut g, 1, 2);

        let setu = vec![1];
        let setv = vec![2];

        let mut matcher = HopcroftKarp::new(g, setu, setv);
        let got = matcher.solve();
        let mat = matcher.get_matching();
        assert_eq!(got, 1);
        let exp: Vec<i32> = vec![-1, 2, 1, -1, -1];
        assert_eq!(*mat, exp);
    }

    #[test]
    fn small() {
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 4];
        add_edge(&mut g, 1, 2);

        let setu = vec![1];
        let setv = vec![2];

        let mut matcher = HopcroftKarp::new(g, setu, setv);
        let got = matcher.solve();
        let mat = matcher.get_matching();
        assert_eq!(got, 1);
        let exp: Vec<i32> = vec![-1, 2, 1, -1, -1];
        assert_eq!(*mat, exp);
    }

    #[test]
    fn small2() {
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 5];
        add_edge(&mut g, 1, 3);

        let setu = vec![1, 2];
        let setv = vec![3];

        let mut matcher = HopcroftKarp::new(g, setu, setv);
        let got = matcher.solve();
        let mat = matcher.get_matching();
        assert_eq!(got, 1);
        let exp: Vec<i32> = vec![-1, 3, -1, 1, -1, -1];
        assert_eq!(*mat, exp);
    }

    #[test]
    #[timeout(1000)]
    fn medium() {
        let edges: Vec<(usize, usize)> =
                    vec![(10, 14), (8, 17), (5, 11), (4, 20), (10, 11),
                         (5, 14), (5, 12), (2, 17), (10, 13), (4, 13),
                         (9, 15), (7, 12), (8, 15), (10, 16), (8, 12),
                         (3, 15), (3, 19), (2, 18), (10, 15), (6, 19),
                         (3, 18), (6, 18), (9, 19), (4, 17), (1, 17)];
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 21];
        for e in edges {
            add_edge(&mut g, e.0, e.1);
        }

        let setu = (1..11).collect();
        let setv = (11..21).collect();

        let mut matcher = HopcroftKarp::new(g, setu, setv);
        let got = matcher.solve();
        let _mat = matcher.get_matching();
        assert_eq!(got, 8);
    }

    #[test]
    fn large_random() {
        let maxn = 250_000;
        let mil = 1_000_000;

        let mut rng = rand::thread_rng();
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 2*maxn];
        for _ in 0..mil {
            let v = rng.gen_range(0..maxn);
            let u = rng.gen_range(maxn..2*maxn);
            add_edge(&mut g, v, u);
        }

        let setu = (0..maxn).collect();
        let setv = (maxn..2*maxn).collect();

        let mut matcher = HopcroftKarp::new(g, setu, setv);
        matcher.solve();
    }
}
