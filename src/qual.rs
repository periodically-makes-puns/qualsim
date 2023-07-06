use std::collections::HashMap;
use std::cmp::{max, min};
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy)]
pub struct State {
    pub time: u8, // 0-89, 7 bits, REMOVE
    pub inner_quiet: u8, // 0-10, 4 bits
    pub cp: u16, // 0-1023, 10 bits
    pub durability: i8, // 0-16, 5 bits
    pub manipulation: i8, // 0-8, 4 bits
    pub waste_not: i8, // 0-8, 4 bits
    pub innovation: i8, // 0-4, 3 bits
    pub great_strides: i8, // 0-3, 2 bits
    pub heart_and_soul: bool, // 1 bit
}

impl State {
    pub fn unpack(st: u64) -> State {
        State { 
            time:           ((st & 0x000001FE00000000) >> 33) as u8, // yes
            inner_quiet:    ((st & 0x00000001E0000000) >> 29) as u8, // 4
            cp:             ((st & 0x000000001FF80000) >> 19) as u16, // 10
            durability:     ((st & 0x000000000007C000) >> 14) as i8, // 5
            manipulation:   ((st & 0x0000000000003C00) >> 10) as i8, // 4
            waste_not:      ((st & 0x00000000000003C0) >> 06) as i8, // 4
            innovation:     ((st & 0x0000000000000038) >> 03) as i8, // 3
            great_strides:  ((st & 0x0000000000000006) >> 01) as i8, // 2
            heart_and_soul: ((st & 0x0000000000000001) != 0)  // 1 
        }
    }

    pub fn index(&self, check_time: bool) -> u64 {
        (self.heart_and_soul as u64) // 1
        + ((self.great_strides as u64) << 1) // 2
        + ((self.innovation as u64) << 3) // 3
        + ((self.waste_not as u64) << 6) // 4
        + ((self.manipulation as u64) << 10) // 4
        + ((self.durability as u64) << 14) // 5
        + ((self.cp as u64) << 19) // 10
        + ((self.inner_quiet as u64) << 29) // 4
        + if check_time {(self.time as u64) << 33} else {0}  // 7
        // the overall space requirement is
        // 90 * 11 * 1000 * 17 * 9 * 9 * 5 * 4 * 2 = 38B
        // too large for an array so a hashmap is best
    }
    
 }

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T: {}, I: {}, CP: {}, D: {}, MANIP{}, WN{}, IN{}, GS{}, HaS? {}", 
            self.time, self.inner_quiet, self.cp, self.durability, self.manipulation, 
            self.waste_not, self.innovation, self.great_strides, self.heart_and_soul)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DPCache {
    cache: HashMap<u64, u64>,
    pub hits: u64,
    pub items: u64,
    check_time: bool,
    max_dur: i8
}


pub fn pack_method(quality: u16, method: u8, state: &State, check_time: bool) -> u64 {
    ((quality as u64) << 48) + ((method as u64) << 40) + state.index(check_time)
}

pub fn unpack_method(packed_result: u64) -> (u16, u8, u64) {
    // quality, method, state
    ((packed_result >> 48) as u16, (packed_result >> 40) as u8, packed_result & ((1 << 40) - 1))
}

pub fn apply_igs(quality: u16, innovation: i8, great_strides: i8, inner_quiet: u8) -> u16 {
    quality * (10 + inner_quiet as u16) / 20 * (2 + (if innovation > 0 {1} else {0}) + (if great_strides > 0 {2} else {0}))
}

pub static UNIT: u16 = 400;

pub static ACTIONS: [&str; 20] = ["(finished)", "",
    "Basic Touch", "Standard Touch", "Advanced Touch", "Basic+Standard", "Advanced Combo", 
    "Focused Touch", "Prudent Touch", "Preparatory Touch", "Trained Finesse", "Waste Not I", 
    "Waste Not II", "Manipulation", "Master's Mend", "Innovation", "Great Strides", 
    "Observe", "Byregot's", "Precise Touch"];

impl DPCache {
    pub fn new(max_dur: i8, check_time: bool) -> DPCache {
        DPCache {
            cache: HashMap::new(),
            hits: 0,
            items: 0,
            check_time,
            max_dur
        }
    }

    pub fn get(&self, index: u64) -> Option<&u64> {
        self.cache.get(&index)
    }

    pub fn insert(&mut self, index: u64, value: u64) -> Option<u64> {
        self.cache.insert(index, value)
    }

    pub fn check(&self, state: &State) -> Option<u64> {
        self.get(state.index(self.check_time)).and_then(|x| Some(*x))
    }

    pub fn query(&mut self, state: &State) -> u64 {
        let index = state.index(self.check_time);
        self.hits += 1;
        match self.get(index) {
            Some(ret) => {return *ret;}
            None => {self.hits -= 1;}
        }
        let State {time, inner_quiet, cp, durability, manipulation, 
            waste_not, innovation, great_strides, heart_and_soul} = state;
        if *cp < 7 || (*time < 2 && self.check_time) { 
            return 0; 
        }
        //println!("EVAL {} {} {} {} {} {} {} {} {}", time, iq, cp, dur, manip, wn, inno, gs, has);
        self.items += 1;
        if self.items % 1000000 == 0 {
            println!("Items: {}", self.items);
        }
        let mut quality_results = [index; 20];
        // instantiate with current statenum to preserve information about remaining resources
        // Basic
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 18 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 18,
                durability: durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet);
            quality_results[1] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 1, &new_state, self.check_time);
        }
        // Standard
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 32 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 32,
                durability: durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT * 5 / 4, *innovation, *great_strides, *inner_quiet);
            quality_results[2] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 2, &new_state, self.check_time);
        }
        // Advanced
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 46 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 46,
                durability: durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT * 3 / 2, *innovation, *great_strides, *inner_quiet);
            quality_results[3] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 3, &new_state, self.check_time);
        }
        // Standard Combo
        if (*durability >= 4 - min(*waste_not, 2) - min(*manipulation, 1)) && *cp >= 36 && (!self.check_time || *time >= 6) {
            let new_state = State {
                time: if self.check_time {time - 6} else {0}, 
                inner_quiet: min(inner_quiet + 2, 10), 
                cp: cp - 36,
                durability: durability - 4 + min(*waste_not, 2) + min(*manipulation, 2),
                manipulation: max(manipulation - 2, 0),
                waste_not: max(waste_not - 2, 0),
                innovation: max(innovation - 2, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet)
                + apply_igs(UNIT * 5 / 4, *innovation - 1, 0, min(*inner_quiet + 1, 10));
            quality_results[4] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 4, &new_state, self.check_time);
        }
        // Advanced Combo
        if (*durability >= 6 - min(*waste_not, 3) - min(*manipulation, 2)) && *cp >= 54 && (!self.check_time || *time >= 9) {
            let new_state = State {
                time: if self.check_time {time - 9} else {0}, 
                inner_quiet: min(inner_quiet + 3, 10), 
                cp: cp - 54,
                durability: durability - 6 + min(*waste_not, 3) + min(*manipulation, 3),
                manipulation: max(manipulation - 3, 0),
                waste_not: max(waste_not - 3, 0),
                innovation: max(innovation - 3, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet)
                + apply_igs(UNIT * 5 / 4, innovation - 1, 0, min(inner_quiet + 1, 10))
                + apply_igs(UNIT * 3 / 2, innovation - 2, 0, min(inner_quiet + 2, 10));
            quality_results[5] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 5, &new_state, self.check_time);
        }
        // Focused Touch
        if (durability + min(*manipulation, 1) >= if *waste_not > 1 {1} else {2}) && *cp >= 25 && (!self.check_time || *time >= 5) {
            let new_state = State {
                time: if self.check_time {time - 5} else {0}, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 25,
                durability: durability - (if *waste_not > 1 {1} else {2}) + min(*manipulation, 2),
                manipulation: max(manipulation - 2, 0),
                waste_not: max(waste_not - 2, 0),
                innovation: max(innovation - 2, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT * 3 / 2, innovation - 1, great_strides - 1, *inner_quiet);
            quality_results[6] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 6, &new_state, self.check_time);
        }
        // Prudent Touch
        if *durability >= 1 && *cp >= 25 && *waste_not == 0 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: min(inner_quiet + 1, 10), 
                cp: cp - 25,
                durability: durability - 1 + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: 0,
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet);
            quality_results[7] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 7, &new_state, self.check_time);
        }
        // Prepratory Touch
        if (*durability >= 4 - (if *waste_not > 0 {2} else {0})) && *cp >= 40 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: min(*inner_quiet + 2, 10), 
                cp: cp - 40,
                durability: durability - 4 + (if *waste_not > 0 {2} else {0}) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT * 2, *innovation, *great_strides, *inner_quiet);
            quality_results[8] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 8, &new_state, self.check_time);
        }
        // Trained Finesse
        if *inner_quiet == 10 && *cp >= 32 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: 10, 
                cp: cp - 32,
                durability: durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT, *innovation, *great_strides, *inner_quiet);
            quality_results[9] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 9, &new_state, self.check_time);
        }
        // Waste Not 1
        if *cp >= 56 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                inner_quiet: *inner_quiet, 
                cp: cp - 56,
                durability: durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: 4,
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            };
            quality_results[10] = pack_method((self.query(&new_state) >> 48) as u16, 10, &new_state, self.check_time);
        }
        // Waste Not 2
        if *cp >= 98 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                inner_quiet: *inner_quiet, 
                cp: cp - 98,
                durability: durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: 8,
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            };
            quality_results[11] = pack_method((self.query(&new_state) >> 48) as u16, 11, &new_state, self.check_time);
        }
        // Manipulation
        if *cp >= 96 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                inner_quiet: *inner_quiet, 
                cp: cp - 96,
                durability: *durability,
                manipulation: 8,
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            };
            quality_results[12] = pack_method((self.query(&new_state) >> 48) as u16, 12, &new_state, self.check_time);
        }
        // Master's Mend
        if *cp >= 88 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                inner_quiet: *inner_quiet, 
                cp: cp - 88,
                durability: *durability + 3 + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            };
            quality_results[13] = pack_method((self.query(&new_state) >> 48) as u16, 13, &new_state, self.check_time);
        }
        // Innovation
        if *cp >= 18 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                inner_quiet: *inner_quiet, 
                cp: cp - 18,
                durability: *durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: 4,
                great_strides: max(great_strides - 1, 0),
                heart_and_soul: *heart_and_soul
            };
            quality_results[14] = pack_method((self.query(&new_state) >> 48) as u16, 14, &new_state, self.check_time);
        }
        // Great Strides
        if *cp >= 32 && (!self.check_time || *time >= 2) {
            let new_state = State {
                time: if self.check_time {time - 2} else {0}, 
                inner_quiet: *inner_quiet, 
                cp: cp - 32,
                durability: *durability + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 3,
                heart_and_soul: *heart_and_soul
            };
            quality_results[15] = pack_method((self.query(&new_state) >> 48) as u16, 15, &new_state, self.check_time);
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
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 24 && *inner_quiet > 0 && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: 0, 
                cp: cp - 24,
                durability: *durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: *heart_and_soul
            };
            let qual = apply_igs(UNIT * (10 + 2 * *inner_quiet as u16) / 10, *innovation, *great_strides, *inner_quiet);
            quality_results[17] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 17, &new_state, self.check_time);
        }
        // Precise Touch
        if (*durability >= 2 - min(*waste_not, 1)) && *cp >= 18 && *heart_and_soul && (!self.check_time || *time >= 3) {
            let new_state = State {
                time: if self.check_time {time - 3} else {0}, 
                inner_quiet: min(inner_quiet + 2, 10), 
                cp: cp - 18,
                durability: *durability - 2 + min(*waste_not, 1) + min(*manipulation, 1),
                manipulation: max(manipulation - 1, 0),
                waste_not: max(waste_not - 1, 0),
                innovation: max(innovation - 1, 0),
                great_strides: 0,
                heart_and_soul: false
            };
            let qual = apply_igs(UNIT * 3 / 2, *innovation, *great_strides, *inner_quiet);
            quality_results[18] = pack_method((self.query(&new_state) >> 48) as u16 + qual, 18, &new_state, self.check_time);
        }
        let ret = *quality_results.iter().max().unwrap();
        self.insert(index, ret);
        ret
    }

    pub fn print_backtrace(&self, state: &State) {
        println!("START {}", State::unpack(state.index(self.check_time)));
        let mut prev = self.check(state).unwrap_or_else(|| 0);
        let (mut qual, mut method, mut last) = unpack_method(prev);
        let mut orig = qual;
        println!("TOTAL: {:.4}", qual as f64 / 400.0);
        while method > 0 {
            assert!(method < 19, "invalid method");
            prev = match self.get(last) {None => 0, Some(t) => *t};
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
            prev = match self.get(last) {None => 0, Some(t) => *t};
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
            curr = match self.get(next) {None => 0, Some(t) => *t};
            (_, method, next) = unpack_method(curr);
        }
        return State::unpack(prev);
    }
}