use crate::hopcroft_karp::HopcroftKarp;
use crate::map::*;
use crate::matching::{Matcher, makespan_solve};

pub trait AgentStrategy {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction>;
}

pub struct MakeSpanHopcroft;
impl AgentStrategy for MakeSpanHopcroft {
    fn pick(&self, map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<Direction> {
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
