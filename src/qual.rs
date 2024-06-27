use std::collections::HashMap;
use std::cmp::{max, min};
use std::fmt;
use std::num::{NonZero, NonZeroU64};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy)]
pub struct State {
    pub time: u8, // 0-89, 7 bits, REMOVE
    pub inner_quiet: u8, // 0-10, 4 bits
    pub cp: u16, // 0-1023, 10 bits
    pub durability: u8, // 0-16, 5 bits
    pub manipulation: u8, // 0-8, 4 bits
    pub waste_not: u8, // 0-8, 4 bits
    pub innovation: u8, // 0-4, 3 bits
    pub great_strides: u8, // 0-3, 2 bits
    pub min_durability: u8, // 0-3, 2 bits
    pub trained_perfection: u8, // 0-2, 2 bits // TODO: Refactor this to combo instead?
    pub heart_and_soul: bool, // 1 bit
}

impl State {
    pub fn unpack(st: u64) -> State {
        State { 
            time:                ((st >> 37) & 0xFF) as u8, // yes
            inner_quiet:         ((st >> 33) & 0xF) as u8, // 4
            cp:                  ((st >> 23) & 0x3FF) as u16, // 10
            durability:          ((st >> 18) & 0x1F) as u8, // 5
            manipulation:        ((st >> 14) & 0xF) as u8, // 4
            waste_not:           ((st >> 10) & 0xF) as u8, // 4
            innovation:          ((st >> 7) & 0x7) as u8, // 3
            great_strides:       ((st >> 5) & 0x3) as u8, // 2
            min_durability:      ((st >> 3) & 0x3) as u8, // 2
            trained_perfection:  ((st >> 1) & 0x3) as u8, // 2
            heart_and_soul:      ((st & 0x1) != 00)       // 1 
        }
    }

    pub fn index(&self, check_time: bool) -> u64 {
        (self.heart_and_soul as u64) // 1
        + ((self.trained_perfection as u64) << 1)
        + ((self.min_durability as u64) << 3)
        + ((self.great_strides as u64) << 5) // 2
        + ((self.innovation as u64) << 7) // 3
        + ((self.waste_not as u64) << 10) // 4
        + ((self.manipulation as u64) << 14) // 4
        + ((self.durability as u64) << 18) // 5
        + ((self.cp as u64) << 23) // 10
        + ((self.inner_quiet as u64) << 33) // 4
        + if check_time {(self.time as u64) << 37} else {0}  // 7
        // the overall space requirement is
        // 90 * 11 * 1000 * 17 * 9 * 9 * 5 * 4 * 2 = 38B
        // too large for an array so a hashmap is best
    }
    
 }

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T: {}, I: {}, CP: {}, D: {}>{}, MANIP{}, WN{}, IN{}, GS{}, TP{}, HaS? {}", 
            self.time, self.inner_quiet, self.cp, self.durability, self.min_durability, self.manipulation, 
            self.waste_not, self.innovation, self.great_strides, self.trained_perfection, self.heart_and_soul)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DPCache {
    cache: Vec<HashMap<u64, u64>>,
    pub hits: u64,
    pub items: u64,
    check_time: bool,
    max_dur: u8
}


pub fn pack_method(quality: u16, method: u8, state: &State, check_time: bool) -> u64 {
    ((quality as u64) << 48) + ((method as u64) << 40) + state.index(check_time)
}

pub fn unpack_method(packed_result: u64) -> (u16, u8, u64) {
    // quality, method, state
    ((packed_result >> 48) as u16, (packed_result >> 40) as u8, packed_result & ((1 << 40) - 1))
}

pub fn apply_igs(quality: u16, innovation: u8, great_strides: u8, inner_quiet: u8, delay: u8) -> u16 {
    quality * (10 + inner_quiet as u16) / 20 * (2 + (if innovation > delay {1} else {0}) + (if great_strides > delay {2} else {0}))
}

pub fn calculate_dur_cost(perstep: u8, steps: u8, delay: u8, wn: u8, manip: u8) -> i8 {
    let wn = max(wn, delay) - delay;

    (perstep*steps) as i8 - (min(manip, steps+delay-1) as i8) - (min(wn, steps)*perstep/2) as i8
}

pub static UNIT: u16 = 400;

pub static ACTION_NAMES: [&str; 23] = ["(finished)", "",
    "Basic Touch", "Standard Touch", "Advanced Touch", "Basic+Standard", "Advanced Combo", 
    "Focused Touch", "Prudent Touch", "Preparatory Touch", "Trained Finesse", "Waste Not I", 
    "Waste Not II", "Manipulation", "Master's Mend", "Innovation", "Great Strides", 
    "Observe", "Byregot's", "Precise Touch", "Basic+Refined", "Immaculate Mend", "Trained Perfection"];
pub struct Action {
    pub name: &'static str,
    pub action_id: u8,
    pub raw_dur_cost: u8,
    pub step_count: u8,
    pub delay: u8,
    pub cp_cost: u16,
    pub iq_stacks: u8,
    pub time_cost: u8,
    pub qual_value: u16,
    pub scaling: u16,
    pub modification: fn(&mut State)
}

pub static ACTIONS: [Action; 20] = [
    Action {
        name: "Basic Touch",
        action_id: 1,
        raw_dur_cost: 2,
        step_count: 1,
        delay: 0,
        cp_cost: 18,
        iq_stacks: 1,
        time_cost: 3,
        qual_value: UNIT,
        scaling: 0,
        modification: |_| {}
    },
    Action {
        name: "Standard Touch",
        action_id: 2,
        raw_dur_cost: 2,
        step_count: 1,
        delay: 0,
        cp_cost: 32,
        iq_stacks: 1,
        time_cost: 3,
        qual_value: UNIT * 5 / 4,
        scaling: 0,
        modification: |_| {}
    },
    Action {
        name: "Advanced Touch",
        action_id: 3,
        raw_dur_cost: 2,
        step_count: 1,
        delay: 0,
        cp_cost: 46,
        iq_stacks: 1,
        time_cost: 3,
        qual_value: UNIT * 3 / 2,
        scaling: 0,
        modification: |_| {}
    },
    Action {
        name: "Basic+Standard",
        action_id: 4,
        raw_dur_cost: 2,
        step_count: 2,
        delay: 0,
        cp_cost: 36,
        iq_stacks: 2,
        time_cost: 6,
        qual_value: UNIT,
        scaling: UNIT / 4,
        modification: |_| {}
    },
    Action {
        name: "Advanced Combo",
        action_id: 5,
        raw_dur_cost: 2, 
        step_count: 3,
        delay: 0,
        cp_cost: 54,
        iq_stacks: 3,
        time_cost: 9,
        qual_value: UNIT,
        scaling: UNIT / 4,
        modification: |_| {}
    },
    Action {
        name: "Focused Touch",
        action_id: 6,
        raw_dur_cost: 2, 
        step_count: 1,
        delay: 1,
        cp_cost: 25,
        iq_stacks: 1,
        time_cost: 6,
        qual_value: UNIT * 3 / 2,
        scaling: 0,
        modification: |_| {}
    },
    Action { // ! NEEDS TO CHECK NO WASTE NOT
        name: "Prudent Touch",
        action_id: 7, 
        raw_dur_cost: 1,
        step_count: 1,
        delay: 0,
        cp_cost: 25,
        iq_stacks: 1,
        time_cost: 3,
        qual_value: UNIT,
        scaling: 0,
        modification: |_| {}
    },
    Action {
        name: "Preparatory Touch",
        action_id: 8,
        raw_dur_cost: 4,
        step_count: 1,
        delay: 0,
        cp_cost: 40,
        iq_stacks: 2,
        time_cost: 3,
        qual_value: 2*UNIT,
        scaling: 0,
        modification: |_| {}
    },
    Action { // ! NEEDS TO CHECK 10 IQ
        name: "Trained Finesse",
        action_id: 9,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 32,
        iq_stacks: 0,
        time_cost: 3,
        qual_value: UNIT,
        scaling: 0,
        modification: |_| {}
    },
    Action {
        name: "Waste Not I",
        action_id: 10,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 56,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.waste_not = 4;}
    },
    Action {
        name: "Waste Not II",
        action_id: 11,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 98,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.waste_not = 8;}
    },
    Action { // ! NEEDS TO UNDO MANIP TICK
        name: "Manipulation",
        action_id: 12,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 96,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.manipulation = 8;}
    },
    Action {
        name: "Master's Mend",
        action_id: 13,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 88,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.durability += 6;}
    },
    Action {
        name: "Innovation",
        action_id: 14,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 18,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.innovation = 4;}
    },
    Action {
        name: "Great Strides",
        action_id: 15,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 32,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.great_strides = 4;}
    },
    Action {
        name: "Byregot's",
        action_id: 17,
        raw_dur_cost: 2,
        step_count: 1,
        delay: 0,
        cp_cost: 24,
        iq_stacks: 0,
        time_cost: 3,
        qual_value: UNIT,
        scaling: UNIT / 10,
        modification: |st| {st.inner_quiet = 0;}
    },
    Action { // ! CHECK HAS
        name: "Precise Touch",
        action_id: 18,
        raw_dur_cost: 2,
        step_count: 1,
        delay: 0,
        cp_cost: 18,
        iq_stacks: 2,
        time_cost: 6,
        qual_value: UNIT * 3 / 2,
        scaling: 0,
        modification: |st: &mut State| {st.heart_and_soul = false;}
    },
    Action {
        name: "Basic+Refined",
        action_id: 19,
        raw_dur_cost: 2,
        step_count: 2,
        delay: 0,
        cp_cost: 42,
        iq_stacks: 3,
        time_cost: 6,
        qual_value: UNIT,
        scaling: 0,
        modification: |_| {}
    },
    Action {
        name: "Immaculate Mend",
        action_id: 20,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 112,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {st.durability = 24;}
    },
    Action {
        name: "Trained Perfection",
        action_id: 21,
        raw_dur_cost: 0,
        step_count: 1,
        delay: 0,
        cp_cost: 0,
        iq_stacks: 0,
        time_cost: 2,
        qual_value: 0,
        scaling: 0,
        modification: |st| {if st.trained_perfection == 0 {st.trained_perfection = 1;}}
    }
];
impl DPCache {
    pub fn new(max_dur: u8, check_time: bool) -> DPCache {
        let mut caches: Vec<HashMap<u64, u64>> = Vec::new();
        for _ in 0..120 { 
            caches.push(HashMap::new());
        }
        DPCache {
            cache: caches,
            hits: 0,
            items: 0,
            check_time,
            max_dur
        }
    }

    pub fn get(&self, time: u8, index: u64) -> Option<&u64> {
        self.cache[if self.check_time {time as usize} else {0}].get(&index)
    }

    pub fn get_state(&self, state: &State) -> Option<&u64> {
        self.cache[if self.check_time {state.time as usize} else {0}].get(&state.index(false))
    }

    pub fn insert(&mut self, time: u8, index: u64, value: u64) -> Option<u64> {
        self.cache[if self.check_time {time as usize} else {0}].insert(index, value)
    }

    pub fn insert_state(&mut self, state: &State, value: u64) -> Option<u64> {
        self.cache[if self.check_time {state.time as usize} else {0}].insert(state.index(false), value)
    }


    pub fn check(&self, state: &State) -> Option<u64> {
        self.get_state(state).and_then(|x| Some(*x))
    }

    pub fn apply_action(&mut self, state: &State, action: &Action) -> Option<NonZero<u64>> {
        let State {time, inner_quiet, cp, durability, manipulation, 
            waste_not, innovation, great_strides, min_durability, trained_perfection, heart_and_soul} = *state;
        let Action {action_id, raw_dur_cost, step_count, delay, cp_cost, iq_stacks, time_cost, qual_value, scaling, modification, ..} = *action;
        if action_id == 7 && waste_not > 0 {return None} 
        if action_id == 9 && inner_quiet != 10 {return None}
        if action_id == 18 && !heart_and_soul {return None}
        if action_id == 21 && trained_perfection == 2 {return None}
        if cp < cp_cost {return None}
        if self.check_time && time < time_cost {return None}
        let dur_cost = 
        if trained_perfection != 1 || action_id == 6 
            {calculate_dur_cost(raw_dur_cost, step_count, delay, waste_not, manipulation)}
        else {calculate_dur_cost(raw_dur_cost, step_count-1, delay+1, waste_not, manipulation)};
        if (durability as i8) < dur_cost {return None}
        let manip_gain = if manipulation >= step_count + delay && action_id != 12 {1} else {0};
        let mut new_state = State {
            time: if self.check_time {time - time_cost} else {0}, 
            inner_quiet: min(inner_quiet + iq_stacks, 10), 
            cp: cp - cp_cost,
            durability: ((durability as i8) + (manip_gain as i8) - dur_cost) as u8,
            manipulation: max(manipulation, step_count + delay) - step_count - delay,
            waste_not: max(waste_not, step_count + delay) - step_count - delay,
            innovation: max(innovation, step_count + delay) - step_count - delay,
            great_strides: if qual_value > 0 {0} else {max(great_strides, step_count + delay) - step_count - delay},
            min_durability,
            trained_perfection: if trained_perfection > 0 {2} else {0},
            heart_and_soul
        };
        modification(&mut new_state);
        new_state.durability = min(new_state.durability, self.max_dur);
        let qual: Option<NonZero<u64>> = self.query(&new_state);
        let qual_value: u16 = if action_id == 17 {UNIT * (10 + 2 * inner_quiet as u16) / 10} else {qual_value};
        if let Some(qual) = qual {
            let qual = qual.get();
            let mut dq = 0u16;
            for item in 0..step_count {
                dq += apply_igs(qual_value + scaling * item as u16, innovation, if item == 0 {great_strides} else {0}, min(inner_quiet+item, 10), delay+item);
            }
            NonZeroU64::new(pack_method(((qual >> 48) as u16) + dq, action_id, &new_state, self.check_time))
        } else {None}
    }

    pub fn query(&mut self, state: &State) -> Option<NonZero<u64>> {
        self.hits += 1;
        match self.get_state(state) {
            Some(ret) => {return NonZeroU64::new(*ret);}
            None => {self.hits -= 1;}
        }
        let State {time, cp, durability,min_durability, ..} = *state;
        if cp < 7 || (time < 2 && self.check_time) { 
            return if durability < min_durability {None} else {NonZeroU64::new(1)}; 
        }
        //println!("EVAL {} {} {} {} {} {} {} {} {}", time, iq, cp, dur, manip, wn, inno, gs, has);
        self.items += 1;
        if self.items % 1000000 == 0 {
            println!("Items: {}", self.items);
        }
        // instantiate with current statenum to preserve information about remaining resources
        // Basic
        let ret = ACTIONS.iter().map(|action| self.apply_action(state, action)).max().flatten();
        self.insert_state(state, if let Some(res) = ret {res.get()} else {0});
        ret
    }

    pub fn unwrapped_query(&mut self, state: &State) -> u64 {
        if let Some(res) = self.query(state) {res.get()} else {0}
    }

    pub fn get_time(ind: u64) -> u8 {
        (ind >> 37) as u8
    }

    pub fn print_backtrace(&self, state: &State) {
        println!("START {}", State::unpack(state.index(self.check_time)));
        let mut prev = self.check(state).unwrap_or_else(|| 0);
        let (mut qual, mut method, mut last) = unpack_method(prev);
        let mut orig = qual;
        println!("TOTAL: {:.4}", qual as f64 / 400.0);
        while method > 0 {
            assert!(method < 22, "invalid method");
            prev = match self.get(Self::get_time(last), last & ((1 << 37) - 1)) {None => 0, Some(t) => *t};
            qual = (prev >> 48) as u16;
            println!("{:02} {:20} {:.4} {}", method, 
                ACTION_NAMES[method as usize + 1], 
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
            assert!(method < 22, "invalid method");
            prev = match self.get(Self::get_time(last), last & ((1 << 33) - 1)) {None => 0, Some(t) => *t};
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
                19 => {
                    println!("/ac \"Basic Touch\" <wait.3>");
                    println!("/ac \"Refined Touch\" <wait.3>");
                }
                20 => {
                    println!("/ac \"Immaculate Mend\" <wait.2>");
                }
                21 => {
                    println!("/ac \"Trained Perfection\" <wait.2>");
                }
                _ => {}
            }
            (_, method, last) = unpack_method(prev);
        }
    }

    pub fn check_endstate(&mut self, st: &State) -> State {
        let mut prev = st.index(self.check_time);
        let mut curr = self.unwrapped_query(st);
        let (_qual,  mut method, mut next) = unpack_method(curr);
        while method > 0 {
            assert!(method < 22, "invalid method");
            prev = next;
            curr = match self.get(Self::get_time(next), next & ((1 << 33) - 1)) {None => 0, Some(t) => *t};
            (_, method, next) = unpack_method(curr);
        }
        return State::unpack(prev);
    }
}