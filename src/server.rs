use crate::qual::{DPCache, State, apply_igs, UNIT, pack_method, unpack_method, ACTIONS};
use serde::{Serialize, Deserialize};
use scc::{TreeIndex, Queue};
use std::cmp::{min, max};
use rayon::prelude::*;
use std::collections::{VecDeque, BTreeSet, BTreeMap};

#[derive(Serialize, Deserialize)]
pub struct AsyncCache {
    cache: TreeIndex<u64, u64>,
    check_time: bool,
    max_dur: i8
}

pub struct Query {
    total_calculations: u64,
    items: BTreeSet<u64>,
    dependencies: TreeIndex<u64, Vec<u64>>,
    unresolved_count: TreeIndex<u64, u8>,
    resolvable: Queue<u64>
}

impl Query {
    pub fn new(cache: &AsyncCache, state: &State) -> Query {
        let mut res = Query {
            total_calculations: 0,
            items: BTreeSet::new(),
            dependencies: TreeIndex::new(),
            dependents: TreeIndex::new(),
            unresolved_count: TreeIndex::new(),
            resolvable: Queue::default()
        };
        let mut calculated: BTreeSet<u64> = BTreeSet::new();
        let mut q: VecDeque<u64> = VecDeque::new();
        let top_index = state.index(cache.check_time);
        q.push_back(top_index);
        res.items.insert(top_index);
        while !q.is_empty() {
            let top = q.pop_front().expect("Queue should be poppable if not empty.");
            res.total_calculations += 1;
            let mut dependencies: Vec<u64> = Vec::new();
            for item in cache.dependencies(&State::unpack(top)) {
                let index = item.0.index(cache.check_time);
                if calculated.contains(&index) {continue;}
                if !res.items.contains(&index) {
                    match cache.prequery(&item.0) {
                        Some(_res) => {calculated.insert(index);}
                        None => {q.push_back(index); res.items.insert(index); dependencies.push(index);}
                    }
                }
            }
            let unresolved = dependencies.len() as u8;
            res.unresolved_count.insert(top, unresolved).expect("BFS should only reach each node once.");
            res.dependencies.insert(top, dependencies).expect("BFS should only reach each node once.");
            if unresolved == 0 {
                res.resolvable.push(top);
            }
        }
        res
    }

    fn resolve(&self, state: &State) {
        self.items.iter().map(|key| {
            if 
        });
    }
}

impl Iterator for Query {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        todo!()
    }
}

impl AsyncCache {
    pub fn new(max_dur: i8, check_time: bool) -> AsyncCache {
        AsyncCache {
            cache: TreeIndex::new(),
            check_time,
            max_dur
        }
    }

    pub fn get(&self, index: u64) -> Option<u64> {
        self.cache.read(&index, |_k, v| *v)
    }

    pub fn insert(&mut self, index: u64, value: u64) {
        self.cache.insert(index, value).unwrap();
    }

    pub fn check(&self, state: &State) -> Option<u64> {
        self.get(state.index(self.check_time)).and_then(|x| Some(x))
    }

    pub fn prequery(&self, state: &State) -> Option<u64> {
        if state.cp < 7 || (state.time < 2 && self.check_time) {return Some(0)}
        let index = state.index(self.check_time);
        self.cache.read(&index, |_k, v| *v)
    }

    pub fn query(&self, state: &State) -> u64 {
        self.prequery(state).unwrap_or_else(|| self.compute(state))
    }

    pub fn dependencies(&self, state: &State) -> Vec<(State, u16, u8)> {
        let State {time, inner_quiet, cp, durability, manipulation, 
            waste_not, innovation, great_strides, heart_and_soul} = state;
        //println!("EVAL {} {} {} {} {} {} {} {} {}", time, inner_quiet, cp, durability, manipulation, waste_not, innovation, great_strides, heart_and_soul);
        //let mut states: [State; 20] = [State::unpack(0); 20]; // used to bring the states into this scope
        let mut jobs: Vec<(State, u16, u8)> = Vec::new();
        // instantiate with current statenum to preserve information about remaining resources
        // Basic
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 18 && *time >= 3 {
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 18,
                durability: durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 1));
        }
        // Standard
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 32 && *time >= 3 {
            let qual = apply_igs(UNIT * 5 / 4, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 32,
                durability: durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 2));
        }
        // Advanced
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 46 && *time >= 3 {
            let qual = apply_igs(UNIT * 3 / 2, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 46,
                durability: durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 3));
        }
        // Standard Combo
        if (*durability >= 4 - min(*waste_not, 2) - min(*manipulation, 1)) && *cp >= 36 && *time >= 6 {
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet)
                + apply_igs(UNIT * 5 / 4, *innovation - 1, 0, min(*inner_quiet + 1, 10));
            jobs.push((State {
                time: time - 6, 
                inner_quiet: min(inner_quiet + 2, 10), 
                cp: cp - 36,
                durability: durability - 4 + min(*waste_not, 2) + min(*manipulation, 2),
                manipulation: max(manipulation - 2, 0),
                waste_not: max(waste_not - 2, 0),
                innovation: max(innovation - 2, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 4));
        }
        // Advanced Combo
        if (*durability >= 6 - min(*waste_not, 3) - min(*manipulation, 2)) && *cp >= 54 && *time >= 9 {
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet)
                + apply_igs(UNIT * 5 / 4, innovation - 1, 0, min(inner_quiet + 1, 10))
                + apply_igs(UNIT * 3 / 2, innovation - 2, 0, min(inner_quiet + 2, 10));
                jobs.push((State {
                    time: time - 9, 
                    inner_quiet: min(inner_quiet + 3, 10), 
                    cp: cp - 54,
                    durability: durability - 6 + min(*waste_not, 3) + min(*manipulation, 3),
                    manipulation: max(manipulation - 3, 0),
                    waste_not: max(waste_not - 3, 0),
                    innovation: max(innovation - 3, 0),
                    great_strides: 0,
                    heart_and_soul: *heart_and_soul
                }, qual, 5));
        }
        // Focused Touch
        if (durability + min(*manipulation, 1) >= if *waste_not > 1 {1} else {2}) && *cp >= 25 && *time >= 5 {
            let qual = apply_igs(UNIT * 3 / 2, innovation - 1, great_strides - 1, *inner_quiet);
            jobs.push((State {
                time: time - 5, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 25,
                durability: durability - (if *waste_not > 1 {1} else {2}) + min(*manipulation, 2),
                manipulation: max(manipulation - 2, 0),
                waste_not: max(waste_not - 2, 0),
                innovation: max(innovation - 2, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 6));
        }
        // Prudent Touch
        if *durability >= 1 && *cp >= 25 && *waste_not == 0 && *time >= 3 {
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 25,
                durability: durability - 1 + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: 0,
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 7));
        }
        // Prepratory Touch
        if (*durability >= 4 - (if *waste_not > 0 {2} else {0})) && *cp >= 40 && *time >= 3 {
            let qual = apply_igs(UNIT * 2, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: min(*inner_quiet + 2, 10), 
                cp: cp - 40,
                durability: durability - 4 + (if *waste_not > 0 {2} else {0}) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 8));
        }
        // Trained Finesse
        if *inner_quiet == 10 && *cp >= 32 && *time >= 3 {
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: 10, 
                cp: cp - 32,
                durability: durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 9));
        }
        // Waste Not 1
        if *cp >= 56 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                inner_quiet: *inner_quiet, 
                cp: cp - 56,
                durability: durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: 4,
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            }, 0, 10));
        }
        // Waste Not 2
        if *cp >= 98 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                inner_quiet: *inner_quiet, 
                cp: cp - 98,
                durability: durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: 8,
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            }, 0, 11));
        }
        // Manipulation
        if *cp >= 96 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                inner_quiet: *inner_quiet, 
                cp: cp - 96,
                durability: *durability,
                manipulation: 8,
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            }, 0, 12));
        }
        // Master's Mend
        if *cp >= 88 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                inner_quiet: *inner_quiet, 
                cp: cp - 88,
                durability: *durability + 3 + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            }, 0, 13));
        }
        // Innovation
        if *cp >= 18 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                inner_quiet: *inner_quiet, 
                cp: cp - 18,
                durability: *durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: 4,
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            }, 0, 14));
        }
        // Great Strides
        if *cp >= 32 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                inner_quiet: *inner_quiet, 
                cp: cp - 32,
                durability: *durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 3,
                heart_and_soul: *heart_and_soul
            }, 0, 15));
        }
        /* Observe
        if *cp >= 7 && *time >= 2 {
            jobs.push((State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 7,
                dur: *dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            }, 0, 16));
        }*/
        // Byregot's Blessing
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 24 && *inner_quiet > 0 && *time >= 3 {
            let qual = apply_igs(UNIT * (10 + 2 * *inner_quiet as u16) / 10, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: 0, 
                cp: cp - 24,
                durability: *durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            }, qual, 17));
        }
        // Precise Touch
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 18 && *heart_and_soul && *time >= 3 {
            let qual = apply_igs(UNIT * 3 / 2, *innovation, *great_strides, *inner_quiet);
            jobs.push((State {
                time: time - 3, 
                inner_quiet: min(inner_quiet + 2, 10), 
                cp: cp - 18,
                durability: *durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: false
            }, qual, 18));
        }
        jobs
    }

    pub fn compute(&self, state: &State) -> u64 {
        let index = state.index(self.check_time);
        //println!("EVAL {} {} {} {} {} {} {} {} {}", time, inner_quiet, cp, durability, manipulation, waste_not, innovation, great_strides, heart_and_soul);
        //let mut states: [State; 20] = [State::unpack(0); 20]; // used to bring the states into this scope
        let mut results: Vec<u64> = Vec::new();
        let mut tasks: Vec<(State, u16, u8)> = Vec::new();
        for job in self.dependencies(state) {
            match self.prequery(&job.0) {
                Some(res) => results.push(pack_method((res >> 48) as u16 + job.1, job.2, &job.0, self.check_time)),
                None => tasks.push(job)
            }
        }
        let res = max(tasks.par_iter().map(|job| pack_method((self.compute(&job.0) >> 48) as u16 + job.1, job.2, &job.0, self.check_time)).max().unwrap_or(0), 
            results.iter().map(|x| *x).max().unwrap_or(0).into());
        
        match self.cache.insert(index, res) {
            Ok(_) => {},
            Err(err) => {
                println!("Failed to insert {} {} {}", err.0, err.1, 
                    self.cache.read(&err.0, |_k, v| *v).unwrap());
            }
        }
        res
    }

    pub fn print_backtrace(&self, state: &State) {
        println!("START {}", State::unpack(state.index(self.check_time)));
        let mut prev = self.check(state).unwrap_or_else(|| 0);
        let (mut qual, mut method, mut last) = unpack_method(prev);
        let mut orig = qual;
        println!("TOTAL: {:.4}", qual as f64 / 400.0);
        while method > 0 {
            assert!(method < 19, "invalid method");
            prev = self.get(last).unwrap_or(0);
            qual = (prev >> 48) as u16;
            println!("{:02} {:20} {:.4} {}", method, 
                ACTIONS[method as usize + 1], 
                (orig - qual) as f64 / 400.0,
                State::unpack(last));
            (orig, method, last) = unpack_method(prev);
        }
        println!("FINISHED");
    }

    pub fn print_macro(&self, st: &State) {
        let mut prev = self.check(st).unwrap_or_else(|| 0);
        let (_, mut method, mut last) = unpack_method(prev);
        while method > 0 {
            assert!(method < 19, "invalid method");
            prev = self.get(last).unwrap_or(0);
            match method {
                1 => {println!("/ac \"Basic Touch\" <wait.3>");},
                2 => {println!("/ac \"Standard Touch\" <wait.3>");},
                3 => {println!("/ac \"Advanced Touch\" <wait.3>");},
                4 => {
                    println!("/ac \"Basic Touch\" <wait.3>");
                    println!("/ac \"Standard Touch\" <wait.3>");
                },
                5 => {
                    println!("/ac \"Basic Touch\" <wait.3>");
                    println!("/ac \"Standard Touch\" <wait.3>");
                    println!("/ac \"Advanced Touch\" <wait.3>");
                },
                6 => {
                    println!("/ac Observe <wait.3>");
                    println!("/ac \"Focused Touch\" <wait.3>");
                },
                7 => {
                    println!("/ac \"Prudent Touch\" <wait.3>");
                }
                8 => {
                    println!("/ac \"Preparatory Touch\" <wait.3>");
                },
                9 => {
                    println!("/ac \"Trained Finesse\" <wait.3>");
                },
                10 => {
                    println!("/ac \"Waste Not\" <wait.2>");
                },
                11 => {
                    println!("/ac \"Waste Not II\" <wait.2>");
                },
                12 => {
                    println!("/ac \"Manipulation\" <wait.2>");
                },
                13 => {
                    println!("/ac \"Master's Mend\" <wait.2>");
                },
                14 => {
                    println!("/ac Innovation <wait.2>");
                }
                15 => {
                    println!("/ac \"Great Strides\" <wait.2>");
                }
                16 => {
                    println!("/ac Observe <wait.3>");
                }
                17 => {
                    println!("/ac \"Byregot's Blessing\" <wait.3>");
                }
                18 => {
                    println!("/ac \"Heart and Soul\" <wait.3>");
                    println!("/ac \"Precise Touch\" <wait.3>");
                }
                _ => {}
            }
            (_, method, last) = unpack_method(prev);
        }
    }

    pub fn check_endstate(&mut self, st: &State) -> State {
        let mut prev = st.index(self.check_time);
        let mut curr = self.query(st);
        let (_qual,  mut method, mut next) = unpack_method(curr);
        while method > 0 {
            assert!(method < 19, "invalid method");
            prev = next;
            curr = self.get(next).unwrap_or(0);
            (_, method, next) = unpack_method(curr);
        }
        return State::unpack(prev);
    }
}