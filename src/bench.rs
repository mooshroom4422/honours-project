use crate::{agent_strategies::*, flow::*, hopcroft_karp::HopcroftKarp, map::*, matching::*, runner::*, target_strategies::*, TurboMatching};
use std::time::Instant;
use rand::{rngs::ThreadRng, Rng};
use tqdm::tqdm;

pub struct AgentStrategyTemplate {
    pub strategy: AgentStrategies,
    pub permutation: Option<Vec<usize>>,
    pub matcher: Option<HopcroftKarp>, // TODO: replace with traits later
    pub flow: Option<FordFulkerson>,
}

impl AgentStrategyTemplate {
    fn construct(&self, map: &Map, agents: &mut Vec<Agent>, targets: &Vec<Target>) -> Box<dyn AgentStrategy> {
        match self.strategy {
            AgentStrategies::MakeSpanHopcroft => Box::new(MakeSpanHopcroft {}),
            AgentStrategies::NoCollisionSingle => {
                let mut res = NoCollisionSingle::new();
                res.prep(map, &agents[0], &targets[0]);
                Box::new(res)
            },
            AgentStrategies::CollisionAssigned => {
                let mut res = CollisionAssigned::new();
                res.prep(map, agents, targets, &self.permutation.clone().unwrap());
                Box::new(res)
            },
            AgentStrategies::CollisionFree => {
                let mut res = CollisionFree::new();
                res.prep(map, agents, targets, &mut self.matcher.clone().unwrap());
                Box::new(res)
            },
            AgentStrategies::NoCollisionFree => {
                let mut res = NoCollisionFree::new();
                res.prep(map, agents, targets, &mut self.flow.clone().unwrap());
                Box::new(res)
            },
        }
    }
}

pub struct TargetStrategyTemplate {
    pub strategy: TargetStrategies,
}

impl TargetStrategyTemplate {
    fn construct(&self, map: &Map, agents: &Vec<Agent>, targets: &mut Vec<Target>) -> Box<dyn TargetStrategy> {
        match self.strategy {
            TargetStrategies::RandomTarget => Box::new(RandomTarget {}),
            TargetStrategies::MaximizeMinDist => Box::new(MaximizeMinDist {}),
            TargetStrategies::TargetFollowPath => {
                // TODO: move true and 100 to config
                let mut res = TargetFollowPath::new(targets.len(), map,
                    targets.iter().map(|x| x.position).collect(), targets, true, 10);
                Box::new(res)
            },
        }
    }
}

pub struct BenchmarkResult {
    pub avg_length: f64,
    pub avg_time: f64,
    pub all_results: Vec<u64>,
}

fn sample(map: &Map, so_far: &Vec<Point>, lu: &Point, rd: &Point, rng: &mut ThreadRng) -> Result<Point, String> {
    if rd.x < lu.x || rd.y < lu.y {
        return Err(format!("invalid rectangle dimensions, lu={:?}, rd={:?}", lu, rd));
    }

    let rect_size = (rd.x-lu.x+1)*(rd.y-lu.y+1);
    let mut iter = 0;
    loop {
        if iter > 2*rect_size {
            return Err(format!("generator gave up after {} iterations. ld={:?}, rd={:?}", iter, lu, rd));
        }

        let x = rng.gen_range(lu.x..=rd.x);
        let y = rng.gen_range(lu.y..=rd.y);
        let pnt = Point{ x, y };

        if map.valid_point(&pnt) && !so_far.contains(&pnt){
            return Ok(pnt);
        }

        iter += 1;
    }
}

pub fn gen_set(map: &Map, nruns: usize, d_time: i32, num_agents: usize, num_targets: usize, rng: &mut ThreadRng,
               agent_rectangles: Vec<(Point, Point)>, target_rectangles: Vec<(Point, Point)>,
              ) -> Result<(Vec<Vec<Agent>>, Vec<Vec<Target>>), String> {

    let mut all_agents: Vec<Vec<Agent>> = Vec::new();
    let mut all_targets: Vec<Vec<Target>> = Vec::new();

    for _iter in 0..nruns {
        let mut generated_so_far: Vec<Point> = Vec::new();
        let mut agent_points: Vec<Point> = Vec::new();
        for i in 0..num_agents {
            let mut lu = Point{ x: 0, y: 0 };
            let mut rd = Point{ x: map.height-1, y: map.width-1 };

            // if possible, sample from rectangle
            if i < agent_rectangles.len() {
                lu = agent_rectangles[i].0;
                rd = agent_rectangles[i].1;
            }

            let pnt = sample(&map, &generated_so_far, &lu, &rd, rng);
            match pnt {
                Ok(p) => {
                    agent_points.push(p);
                    generated_so_far.push(p);
                },
                Err(s) => return Err(s),
            }
        }

        let mut target_points: Vec<Point> = Vec::new();
        for i in 0..num_targets {
            let mut lu = Point{ x: 0, y: 0 };
            let mut rd = Point{ x: map.height-1, y: map.width-1 };

            if i < target_rectangles.len() {
                lu = agent_rectangles[i].0;
                rd = agent_rectangles[i].1;
            }

            let pnt = sample(&map, &generated_so_far, &lu, &rd, rng);
            match pnt {
                Ok(p) => {
                    target_points.push(p);
                    generated_so_far.push(p);
                },
                Err(s) => return Err(s),
            }
        }

        all_agents.push(agents_from(&agent_points));
        all_targets.push(targets_from(&target_points, d_time as i32));
    }

    Ok((all_agents, all_targets))
}

pub fn bench(map: Map, num_runs: i32, d_time: i32, all_agents: Vec<Vec<Agent>>, all_targets: Vec<Vec<Target>>,
             agent_strat_template: AgentStrategyTemplate, target_strat: &mut Vec<Box<dyn TargetStrategy>>,
             debug_print: bool, collect_individual: bool
            ) -> Result<BenchmarkResult, String> {

    let mut sum_length: u64 = 0;
    let mut sum_time: u128 = 0;
    let mut all_results = Vec::new();
    for run_id in tqdm(0..num_runs as usize) {
        let start_time = Instant::now();

        let mut agents = all_agents[run_id].clone();
        let targets = all_targets[run_id].clone();

        let agent_strat = agent_strat_template.construct(&map, &mut agents, &targets);

        let mut runner = Runner {
            map: map.clone(),
            agents,
            targets,
            d_time
        };

        // println!("starting runner: {}", iter);
        // println!("{:?}", agents);
        // println!("{:?}", targets);

        let took_steps = runner.run(agent_strat, &mut target_strat[run_id], debug_print, false, false, debug_print, "") as u64;

        //println!("run: {} -> {:?} {:?} {:?}", iter, num_agents, num_targets, took_steps);

        sum_time += start_time.elapsed().as_millis();
        sum_length += took_steps;

        if collect_individual {
            all_results.push(took_steps);
        }
    }

    let avg_length: f64 = (sum_length as f64)/(num_runs as f64);
    let avg_time: f64 = (sum_time as f64)/(num_runs as f64);
    if debug_print {
        println!("avg length: {:.4}", avg_length);
        println!("avg time: {:.4}ms", avg_time);
    }

    return Ok(
        BenchmarkResult {
            avg_length,
            avg_time,
            all_results,
        }
    );
}
