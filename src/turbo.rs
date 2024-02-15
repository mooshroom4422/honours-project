use crate::matching::Matcher;

pub struct TurboMatching {
    pub g: Vec<Vec<usize>>,
    pub vis: Vec<bool>,
    pub mat: Vec<i32>,
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
    fn new(graph: Vec<Vec<usize>>, _setu: Vec<usize>, _setv: Vec<usize>) -> Self {
        let n: usize = graph.len();
        TurboMatching {
            g: graph,
            vis: vec![false; n],
            mat: vec![-1; n],
        }
    }

    fn new_empty() -> Self {
        TurboMatching {
            g: Vec::new(),
            vis: Vec::new(),
            mat: Vec::new(),
        }
    }

    fn init(&mut self, graph: Vec<Vec<usize>>, _setu: Vec<usize>, _setv: Vec<usize>) {
        self.g = graph;
        let n = self.g.len();
        self.vis = vec![false; n];
        self.mat = vec![-1; n];
    }

    fn solve(&mut self) -> usize {
        self.matching()
    }

    fn get_matching(&mut self) -> &Vec<i32> {
        &self.mat
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::turbo::TurboMatching;
    use crate::matching::Matcher;

    fn add_edge(g: &mut Vec<Vec<usize>>, v: usize, u: usize) {
        g[v].push(u);
        g[u].push(v);
    }

    #[test]
    fn simple() {
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 4];
        add_edge(&mut g, 1, 2);

        let mut matcher = TurboMatching::new(g, Vec::new(), Vec::new());
        let got = matcher.solve();
        let mat = matcher.get_matching();
        assert_eq!(got, 1);
        let exp: Vec<i32> = vec![-1, 2, 1, -1];
        assert_eq!(*mat, exp);
    }

    #[test]
    fn small() {
        let mut g: Vec<Vec<usize>> = vec![Vec::new(); 4];
        add_edge(&mut g, 1, 2);

        let mut matcher = TurboMatching::new(g, Vec::new(), Vec::new());
        let got = matcher.solve();
        let mat = matcher.get_matching();
        assert_eq!(got, 1);
        let exp: Vec<i32> = vec![-1, 2, 1, -1];
        assert_eq!(*mat, exp);
    }

    #[test]
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

        let mut matcher = TurboMatching::new(g, Vec::new(), Vec::new());
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

        let mut matcher = TurboMatching::new(g, Vec::new(), Vec::new());
        matcher.solve();
    }
}
