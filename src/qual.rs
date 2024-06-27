use std::collections::HashMap;
use std::cmp::{max, min};
use std::fmt;
use std::num::{NonZero, NonZeroU64};
use std::os::raw;
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

pub fn apply_igs(quality: u16, innovation: u8, great_strides: u8, inner_quiet: u8) -> u16 {
    quality * (10 + inner_quiet as u16) / 20 * (2 + (if innovation > 0 {1} else {0}) + (if great_strides > 0 {2} else {0}))
}

pub fn calculate_dur_cost(perstep: u8, steps: u8, delay: u8, wn: u8, manip: u8) -> i8 {
    if steps == 0 {return 0}
    let wn = max(wn, delay) - delay;

    (perstep*steps) as i8 - (min(manip, steps+delay-1) as i8) - (min(wn, steps)*perstep/2) as i8
}

pub static UNIT: u16 = 400;

pub static ACTION_NAMES: [&str; 23] = ["(finished)", "",
    "Basic Touch", "Standard Touch", "Advanced Touch", "Basic+Standard", "Advanced Combo", 
    "Focused Touch", "Prudent Touch", "Preparatory Touch", "Trained Finesse", "Waste Not I", 
    "Waste Not II", "Manipulation", "Master's Mend", "Innovation", "Great Strides", 
    "Observe", "Byregot's", "Precise Touch", "Basic+Refined", "Immaculate Mend", "Trained Perfection"];

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

    pub fn query(&mut self, state: &State) -> Option<NonZero<u64>> {
        let index = state.index(self.check_time);
        self.hits += 1;
        match self.get_state(state) {
            Some(ret) => {return NonZeroU64::new(*ret);}
            None => {self.hits -= 1;}
        }
        let State {time, inner_quiet, cp, durability, manipulation, 
            waste_not, innovation, great_strides, min_durability, trained_perfection, heart_and_soul} = *state;
        if cp < 7 || (time < 2 && self.check_time) { 
            return if durability < min_durability {None} else {NonZeroU64::new(1)}; 
        }
        //println!("EVAL {} {} {} {} {} {} {} {} {}", time, iq, cp, dur, manip, wn, inno, gs, has);
        self.items += 1;
        if self.items % 1000000 == 0 {
            println!("Items: {}", self.items);
        }
        let mut quality_results = [NonZeroU64::new(index); 23];
        // instantiate with current statenum to preserve information about remaining resources
        // Basic
        let action_id: u8 = 1;
        let raw_dur_cost = 2;
        let step_count = 1;
        let delay = 0;
        let cp_cost = 18;
        let iq_stacks = 1;
        let time_cost = 3;
        let qual_value = UNIT;
        let scaling = UNIT / 4;
        let dur_cost = if trained_perfection == 1 {calculate_dur_cost(raw_dur_cost, step_count-delay+1, delay+1, waste_not, manipulation)} 
            else {calculate_dur_cost(raw_dur_cost, step_count-delay, delay, waste_not, manipulation)};
        if durability as i8 >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = if manipulation >= step_count + delay {1} else {0};
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Standard
        let action_id: u8 = 2;
        let raw_dur_cost = 2;
        let step_count = 1;
        let cp_cost = 32;
        let iq_stacks = 1;
        let time_cost = 3;
        let qual_value = UNIT * 5 / 4;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Advanced
        let action_id: u8 = 3;
        let raw_dur_cost = 2;
        let step_count = 1;
        let cp_cost = 46;
        let iq_stacks = 1;
        let time_cost = 3;
        let qual_value = UNIT * 3 / 2;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Standard Combo
        let action_id: u8 = 4;
        let raw_dur_cost = 4;
        let step_count = 2;
        let cp_cost = 36;
        let iq_stacks = 2;
        let time_cost = 6;
        let qual_value = UNIT;
        let dur_cost = if trained_perfection == 1 {raw_dur_cost} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet)
                + apply_igs(UNIT * 5 / 4, max(innovation, 1) - 1, 0, min(inner_quiet+1, 10));
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Advanced Combo
        let action_id: u8 = 5;
        let raw_dur_cost = 6;
        let step_count = 3;
        let cp_cost = 54;
        let iq_stacks = 3;
        let time_cost = 9;
        let qual_value = UNIT;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet)
                + apply_igs(UNIT * 5 / 4, max(innovation, 1) - 1, 0, min(inner_quiet + 1, 10))
                + apply_igs(UNIT * 3 / 2, max(innovation, 2) - 2, 0, min(inner_quiet + 2, 10));
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Focused Touch
        let action_id: u8 = 6;
        let raw_dur_cost = 2;
        let step_count = 2;
        let cp_cost = 25;
        let iq_stacks = 1;
        let time_cost = 6;
        let qual_value = UNIT * 3 / 2;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, max(innovation, 1)-1, max(great_strides, 1) - 1, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Prudent Touch
        let action_id: u8 = 7;
        let raw_dur_cost = 1;
        let step_count = 1;
        let cp_cost = 25;
        let iq_stacks = 1;
        let time_cost = 3;
        let qual_value = UNIT;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Prepratory Touch
        let action_id: u8 = 8;
        let raw_dur_cost = 4;
        let step_count = 1;
        let cp_cost: u16 = 40;
        let iq_stacks = 2;
        let time_cost = 3;
        let qual_value = UNIT * 2;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Trained Finesse
        let action_id: u8 = 9;
        let step_count = 1;
        let cp_cost = 32;
        let time_cost = 3;
        let qual_value = UNIT;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = 0;
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet,
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Waste Not 1
        let action_id: u8 = 10;
        let step_count = 1;
        let cp_cost = 56;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability: min(durability + min(manipulation, step_count), self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: 4,
                innovation: max(innovation, step_count) - step_count,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Waste Not 2
        let action_id: u8 = 11;
        let step_count = 1;
        let cp_cost = 98;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability: min(durability + min(manipulation, step_count), self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: 8,
                innovation: max(innovation, step_count) - step_count,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Manipulation
        let action_id: u8 = 12;
        let step_count = 1;
        let cp_cost = 96;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability,
                manipulation: 8,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Master's Mend
        let action_id: u8 = 13;
        let step_count = 1;
        let cp_cost = 88;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability: min(durability + 6 + min(manipulation, step_count), self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Innovation
        let action_id: u8 = 14;
        let step_count = 1;
        let cp_cost: u16 = 18;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability: min(durability + min(manipulation, step_count), self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: 4,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Great Strides
        let action_id: u8 = 15;
        let step_count = 1;
        let cp_cost = 32;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability: min(durability + min(manipulation, step_count), self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 3,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        /* Observe
        if *cp >= 7 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                iq: *iq, 
                cp: cp - 7,
                dur: *dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            };
            res[16] = combine_method((self.query(&new_state) >> 48) as u16, 16, &new_state, self.check_time);
        }*/
        // Byregot's Blessing
        let action_id: u8 = 17;
        let raw_dur_cost = 2;
        let step_count = 1;
        let cp_cost: u16 = 24;
        let time_cost = 3;
        let qual_value = UNIT * (10 + 2 * inner_quiet as u16) / 10;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: 0, 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        // Precise Touch
        let action_id: u8 = 18;
        let raw_dur_cost = 2;
        let step_count = 1;
        let cp_cost = 18;
        let iq_stacks = 2;
        let time_cost = 3;
        let qual_value = UNIT * 2;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && heart_and_soul && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul: false
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet);
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        let action_id: u8 = 19;
        let raw_dur_cost = 4;
        let step_count = 2;
        let cp_cost = 42;
        let iq_stacks = 3;
        let time_cost = 6;
        let qual_value = UNIT;
        let dur_cost = if trained_perfection == 1 {0} 
            else {raw_dur_cost - min(manipulation, step_count-1) - min(waste_not, raw_dur_cost / 2)};
        if durability >= dur_cost && cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let adjusted_cost = if trained_perfection == 1 {0} else {raw_dur_cost - min(waste_not, raw_dur_cost / 2)};
            let manip_gain = min(manipulation, step_count);
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet: min(inner_quiet + iq_stacks, 10), 
                cp: cp - cp_cost,
                durability: min(durability + manip_gain - adjusted_cost, self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: 0,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let qual = apply_igs(qual_value, innovation, great_strides, inner_quiet)
                + apply_igs(UNIT, max(innovation, 1) - 1, 0, min(inner_quiet+1, 10));
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16 + qual, action_id, &new_state, self.check_time))
            } else {None};
        }
        let action_id: u8 = 20;
        let step_count = 1;
        let cp_cost = 112;
        let time_cost = 2;
        if cp >= cp_cost && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp: cp - cp_cost,
                durability: self.max_dur,
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: if trained_perfection > 0 {2} else {0},
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        let action_id: u8 = 21;
        let step_count = 1;
        let time_cost = 2;
        if trained_perfection == 0 && (!self.check_time || time >= time_cost) {
            let new_state = State {
                time: if self.check_time {time - time_cost} else {0}, 
                inner_quiet, 
                cp,
                durability: min(durability + min(manipulation, step_count), self.max_dur),
                manipulation: max(manipulation, step_count) - step_count,
                waste_not: max(waste_not, step_count) - step_count,
                innovation: max(innovation, step_count) - step_count,
                great_strides: max(great_strides, step_count) - step_count,
                min_durability,
                trained_perfection: 1,
                heart_and_soul
            };
            let res = self.query(&new_state);
            quality_results[action_id as usize] = if let Some(res) = res {
                NonZeroU64::new(pack_method((res.get() >> 48) as u16, action_id, &new_state, self.check_time))
            } else {None};
        }
        let ret = *quality_results.iter().max().unwrap();
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
            assert!(method < 19, "invalid method");
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