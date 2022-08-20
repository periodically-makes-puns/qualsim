use std::collections::HashMap;
use std::cmp::{max, min};
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct State {
    pub time: u8, // 0-89, 7 bits
    pub iq: u8, // 0-10, 4 bits
    pub cp: u16, // 0-699, 10 bits
    pub dur: i8, // 0-16, 5 bits
    pub manip: i8, // 0-8, 4 bits
    pub wn: i8, // 0-8, 4 bits
    pub inno: i8,
    pub gs: i8, 
    pub has: bool
}

impl State {
    pub fn unpack(st: u64) -> State {
        State { 
            time:  ((st & 0x000001FE00000000) >> 33) as u8, // yes
            iq:    ((st & 0x00000001E0000000) >> 29) as u8, // 4
            cp:    ((st & 0x000000001FF80000) >> 19) as u16, // 10
            dur:   ((st & 0x000000000007C000) >> 14) as i8, // 5
            manip: ((st & 0x0000000000003C00) >> 10) as i8, // 4
            wn:    ((st & 0x00000000000003C0) >> 06) as i8, // 4
            inno:  ((st & 0x0000000000000038) >> 03) as i8, // 3
            gs:    ((st & 0x0000000000000006) >> 01) as i8, // 2
            has:   ((st & 0x0000000000000001) != 0)  // 1 
        }
    }

    pub fn index(&self) -> u64 {
        (self.has as u64) // 1
        + ((self.gs as u64) << 1) // 2
        + ((self.inno as u64) << 3) // 3
        + ((self.wn as u64) << 6) // 4
        + ((self.manip as u64) << 10) // 4
        + ((self.dur as u64) << 14) // 5
        + ((self.cp as u64) << 19) // 10
        + ((self.iq as u64) << 29) // 4
        + ((self.time as u64) << 33) // 7
        // the overall space requirement is
        // 90 * 11 * 700 * 17 * 9 * 9 * 5 * 4 * 2 = 38B
        // too large for an array so a hashmap is best
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T: {}, I: {}, CP: {}, D: {}, MANIP{}, WN{}, IN{}, GS{}, HaS? {}", 
            self.time, self.iq, self.cp, self.dur, self.manip, self.wn, self.inno, self.gs, self.has)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DPCache {
    cache: HashMap<u64, u64>,
    pub hits: u64,
    pub items: u64
}


fn combine_method(qual: u16, method: u8, state: &State) -> u64 {
    ((qual as u64) << 48) + ((method as u64) << 40) + state.index()
}

pub fn unpack_method(res: u64) -> (u16, u8, u64) {
    ((res >> 48) as u16, (res >> 40) as u8, res & ((1 << 40) - 1))
}

fn apply_igs(qual: u16, inno: i8, gs: i8, iq: u8) -> u16 {
    qual * (10 + iq as u16) / 20 * (2 + (if inno > 0 {1} else {0}) + (if gs > 0 {2} else {0}))
}

pub static UNIT: u16 = 400;

static ACTIONS: [&str; 20] = ["(finished)", "",
    "Basic Touch", "Standard Touch", "Advanced Touch", "Basic+Standard", "Advanced Combo", 
    "Focused Touch", "Prudent Touch", "Preparatory Touch", "Trained Finesse", "Waste Not I", 
    "Waste Not II", "Manipulation", "Master's Mend", "Innovation", "Great Strides", 
    "Observe", "Byregot's", "Precise Touch"];

impl DPCache {
    pub fn new() -> DPCache {
        DPCache {
            cache: HashMap::new(),
            hits: 0,
            items: 0
        }
    }

    pub fn get(&self, ind: u64) -> Option<&u64> {
        self.cache.get(&ind)
    }

    pub fn insert(&mut self, ind: u64, res: u64) -> Option<u64> {
        self.cache.insert(ind, res)
    }

    pub fn query(&mut self, st: &State) -> u64 {
        let ind = st.index();
        self.hits += 1;
        match self.get(ind) {
            Some(ret) => {return *ret;}
            None => {self.hits -= 1;}
        }
        let State {time, iq, cp, dur, manip, wn, inno, gs, has} = st;
        if *cp < 7 || *time < 2 { 
            return 0; 
        }
        //println!("EVAL {} {} {} {} {} {} {} {} {}", time, iq, cp, dur, manip, wn, inno, gs, has);
        self.items += 1;
        if self.items % 1000000 == 0 {
            println!("Items: {}", self.items);
        }
        let mut res = [0; 20];
        // Basic
        if (*dur >= 2 - min(*wn, 1)) && *cp >= 18 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: min(iq + 1, 10), 
                cp: cp - 18,
                dur: dur - 2 + min(*wn, 1) + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT, *inno, *gs, *iq);
            res[1] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 1, &new_state);
        }
        // Standard
        if (*dur >= 2 - min(*wn, 1)) && *cp >= 32 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: min(iq + 1, 10), 
                cp: cp - 32,
                dur: dur - 2 + min(*wn, 1) + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT * 5 / 4, *inno, *gs, *iq);
            res[2] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 2, &new_state);
        }
        // Advanced
        if (*dur >= 2 - min(*wn, 1)) && *cp >= 46 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: min(iq + 1, 10), 
                cp: cp - 46,
                dur: dur - 2 + min(*wn, 1) + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT * 3 / 2, *inno, *gs, *iq);
            res[3] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 3, &new_state);
        }
        // Standard Combo
        if (*dur >= 4 - min(*wn, 2) - min(*manip, 1)) && *cp >= 36 && *time >= 6 {
            let new_state = State {
                time: time - 6, 
                iq: min(iq + 2, 10), 
                cp: cp - 36,
                dur: dur - 4 + min(*wn, 2) + min(*manip, 2),
                manip: max(manip - 2, 0),
                wn: max(wn - 2, 0),
                inno: max(inno - 2, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT, *inno, *gs, *iq)
                + apply_igs(UNIT * 5 / 4, *inno - 1, 0, min(*iq + 1, 10));
            res[4] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 4, &new_state);
        }
        // Advanced Combo
        if (*dur >= 6 - min(*wn, 3) - min(*manip, 2)) && *cp >= 54 && *time >= 9 {
            let new_state = State {
                time: time - 9, 
                iq: min(iq + 3, 10), 
                cp: cp - 54,
                dur: dur - 6 + min(*wn, 3) + min(*manip, 3),
                manip: max(manip - 3, 0),
                wn: max(wn - 3, 0),
                inno: max(inno - 3, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT, *inno, *gs, *iq)
                + apply_igs(UNIT * 5 / 4, inno - 1, 0, min(iq + 1, 10))
                + apply_igs(UNIT * 3 / 2, inno - 2, 0, min(iq + 2, 10));
            res[5] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 5, &new_state);
        }
        // Focused Touch
        if (dur + min(*manip, 1) >= if *wn > 1 {1} else {2}) && *cp >= 25 && *time >= 5 {
            let new_state = State {
                time: time - 5, 
                iq: min(iq + 1, 10), 
                cp: cp - 25,
                dur: dur - (if *wn > 1 {1} else {2}) + min(*manip, 2),
                manip: max(manip - 2, 0),
                wn: max(wn - 2, 0),
                inno: max(inno - 2, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT * 3 / 2, inno - 1, gs - 1, *iq);
            res[6] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 6, &new_state);
        }
        // Prudent
        if *dur >= 1 && *cp >= 25 && *wn == 0 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: min(iq + 1, 10), 
                cp: cp - 25,
                dur: dur - 1 + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: 0,
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT, *inno, *gs, *iq);
            res[7] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 7, &new_state);
        }
        // Prep
        if (*dur >= 4 - (if *wn > 0 {2} else {0})) && *cp >= 40 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: min(*iq + 2, 10), 
                cp: cp - 40,
                dur: dur - 4 + (if *wn > 0 {2} else {0}) + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT * 2, *inno, *gs, *iq);
            res[8] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 8, &new_state);
        }
        // Trained Finesse
        if *iq == 10 && *cp >= 32 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: 10, 
                cp: cp - 32,
                dur: dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT, *inno, *gs, *iq);
            res[9] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 9, &new_state);
        }
        // wn1
        if *cp >= 56 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 56,
                dur: dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: 4,
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            };
            res[10] = combine_method((self.query(&new_state) >> 48) as u16, 10, &new_state);
        }
        // wn2
        if *cp >= 98 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 98,
                dur: dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: 8,
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            };
            res[11] = combine_method((self.query(&new_state) >> 48) as u16, 11, &new_state);
        }
        // manip
        if *cp >= 96 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 96,
                dur: *dur,
                manip: 8,
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            };
            res[12] = combine_method((self.query(&new_state) >> 48) as u16, 12, &new_state);
        }
        // MM
        if *cp >= 88 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 88,
                dur: *dur + 3 + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            };
            res[13] = combine_method((self.query(&new_state) >> 48) as u16, 13, &new_state);
        }
        // inno
        if *cp >= 18 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 18,
                dur: *dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: 4,
                gs: max(gs - 1, 0),
                has: *has
            };
            res[14] = combine_method((self.query(&new_state) >> 48) as u16, 14, &new_state);
        }
        //gs
        if *cp >= 32 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 32,
                dur: *dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 3,
                has: *has
            };
            res[15] = combine_method((self.query(&new_state) >> 48) as u16, 15, &new_state);
        }
        /*
        if *cp >= 7 && *time >= 2 {
            let new_state = State {
                time: time - 2, 
                iq: *iq, 
                cp: cp - 7,
                dur: *dur + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: max(gs - 1, 0),
                has: *has
            };
            res[16] = combine_method((self.query(&new_state) >> 48) as u16, 16, &new_state);
        }*/
        // byregot
        if (*dur >= 2 - min(*wn, 1)) && *cp >= 24 && *iq > 0 && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: 0, 
                cp: cp - 24,
                dur: *dur - 2 + min(*wn, 1) + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: *has
            };
            let qual = apply_igs(UNIT * (10 + 2 * *iq as u16) / 10, *inno, *gs, *iq);
            res[17] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 17, &new_state);
        }
        // precise
        if (*dur >= 2 - min(*wn, 1)) && *cp >= 18 && *has && *time >= 3 {
            let new_state = State {
                time: time - 3, 
                iq: min(iq + 2, 10), 
                cp: cp - 18,
                dur: *dur - 2 + min(*wn, 1) + min(*manip, 1),
                manip: max(manip - 1, 0),
                wn: max(wn - 1, 0),
                inno: max(inno - 1, 0),
                gs: 0,
                has: false
            };
            let qual = apply_igs(UNIT * 3 / 2, *inno, *gs, *iq);
            res[18] = combine_method((self.query(&new_state) >> 48) as u16 + qual, 18, &new_state);
        }
        let ret = *res.iter().max().unwrap();
        self.insert(ind, ret);
        ret
    }

    pub fn print_backtrace(&mut self, st: &State) {
        println!("START {}", State::unpack(st.index()));
        let mut prev = self.query(st);
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

    pub fn check_time(&mut self, st: &State) -> u8{
        let mut prev = st.index();
        let mut curr = self.query(st);
        let (_qual,  mut method, mut next) = unpack_method(curr);
        while method > 0 {
            assert!(method < 19, "invalid method");
            prev = next;
            curr = match self.get(next) {None => 0, Some(t) => *t};
            (_, method, next) = unpack_method(curr);
        }
        return State::unpack(prev).time;
    }
}