#[derive(Debug, Clone)]
pub struct State {
    pub time: u8, // 0-89, 7 bits
    pub inner_quiet: u8, // 0-10, 4 bits
    pub cp: u16, // 0-699, 10 bits
    pub durability: u8, // 0-16, 5 bits
    pub manipulation: u8, // 0-8, 4 bits
    pub waste_not: u8, // 0-8, 4 bits
    pub veneration: u8,
    pub muscle_memory: u8, 
    pub heart_and_soul: bool,
    pub reflect: bool,
    pub progress: u16,
    pub trained_perfection: u8
}



mod actions {
    #[derive(PartialEq)]
    pub enum Status {
        None,
        Manipulation,
        WasteNot,
        Veneration,
        MuscleMemory,
        TrainedPerfection
    }
    pub struct Action {
        pub progress: u16, // Efficiency x10
        pub durability: u8, // 5dur = 1
        pub cp: u16,
        pub status: Status,
        pub duration: u8
    }

    impl Action {
        pub const fn new(progress: u16, durability: u8, cp: u16, status: Status, duration: u8) -> Action {
            Action {
                progress,
                durability,
                cp,
                status,
                duration
            }
        }
    }

	pub const BASIC: Action = Action::new(12, 2, 0, Status::None, 0 );
	pub const CAREFUL: Action = Action::new(18, 2, 7, Status::None, 0 );
	pub const FOCUSED: Action = Action::new(20, 2, 12, Status::None, 0 );
	pub const PRUDENT: Action = Action::new(18, 1, 18, Status::None, 0 );
	pub const GROUNDWORK: Action = Action::new(36, 4, 18, Status::None, 0 );
	pub const MUMEN: Action = Action::new(30, 2, 6, Status::MuscleMemory, 5 );
	pub const VENER: Action = Action::new(0, 0, 18, Status::Veneration, 4 );
	pub const MANIPULATION: Action = Action::new(0, 0, 96, Status::Manipulation, 8 );
	pub const WN1: Action = Action::new(0, 0, 56, Status::WasteNot, 4 );
	pub const WN2: Action = Action::new(0, 0, 98, Status::WasteNot, 8 );
	pub const INTENSIVE: Action = Action::new(40, 2, 6, Status::None, 0 );
    pub const TP: Action = Action::new(0, 0, 0, Status::TrainedPerfection, 0);
}

impl State {
    pub fn apply_opener(&mut self, opener: &str, extra: char) {
        for c in opener.chars() {
            self.apply_char(c)
        }
        self.apply_char(extra)
    }

    pub fn apply_char(&mut self, c: char) {
        if c == 'R' {
            self.durability -= 2;
            self.cp -= 18;
            self.inner_quiet += 2;
            self.reflect = true;
            self.time += 3;
            return;
        }
        if c == ' ' {return;} // noop
        self.apply_action(match c {
            'b' => &actions::BASIC,
            'c' => &actions::CAREFUL,
            'f' => &actions::FOCUSED,
            'p' => &actions::PRUDENT,
            'g' => &actions::GROUNDWORK,
            'M' => &actions::MUMEN,
            'v' => &actions::VENER,
            'm' => &actions::MANIPULATION,
            '1' => &actions::WN1,
            '2' => &actions::WN2,
            'i' => &actions::INTENSIVE,
            '*' => &actions::TP,
            _ => {panic!("Bad action char");}
        });
    }

    pub fn apply_action(&mut self, act: &actions::Action) {
        let w = self.waste_not;
	    let v = self.veneration;
	    let m = self.manipulation;
	    let d = self.durability;
        if act.cp == 12 {
            self.tick_statuses(true);
        }
        if ((if self.trained_perfection == 1 {0} else {act.durability}) > self.durability * (if self.waste_not > 0 {2} else {1})) || act.cp > self.cp || (act.durability == 1 && self.waste_not == 0) {
            self.waste_not = w;
            self.veneration = v;
            self.manipulation = m;
            self.durability = d;
            return;
        }
        self.durability -= (if self.trained_perfection == 1 {0} else {act.durability}) >> (if self.waste_not > 0 {1} else {0});
	    self.cp -= act.cp;
	    let mut action_progress = act.progress;
        if action_progress == 40 {self.heart_and_soul = true;}
        if self.veneration > 0 {action_progress += act.progress / 2;}
        if self.muscle_memory > 0 && action_progress > 0 {
            action_progress += act.progress;
            self.muscle_memory = 0;
        }
        if action_progress > 0 {
            if act.progress == 20 {self.time += 2;}
            self.time += 3;
        } else {
            self.time += 2;
        }
        self.progress += action_progress;
        let tick_manip = act.status != actions::Status::Manipulation;
        self.tick_statuses(tick_manip);
        match act.status {
            actions::Status::Manipulation => {self.manipulation = 8;}
            actions::Status::WasteNot => {self.waste_not = act.duration;}
            actions::Status::Veneration => {self.veneration = 4;}
            actions::Status::MuscleMemory => {self.muscle_memory = 5;}
            actions::Status::TrainedPerfection => {if self.trained_perfection == 0 {self.trained_perfection = 1;}}
            _ => {}
        }
    }

    pub fn tick_statuses(&mut self, tick_manip: bool) {
        if self.waste_not > 0 {self.waste_not -= 1;}
        if self.veneration > 0 {self.veneration -= 1;}
        if self.muscle_memory > 0 {self.muscle_memory -= 1;}
        if self.manipulation > 0 && tick_manip {
            self.manipulation -= 1;
            self.durability += 1;
        }
        if self.trained_perfection > 0 {self.trained_perfection = 2;}
    }
}
#[derive(Debug, PartialEq)]
pub struct Finisher<'a> {
    pub time: u8,
    pub cp: u16,
    pub durability: u8,
    pub progress: u16,
    pub heart_and_soul: bool,
    pub uses_trained_perfection: bool,
    pub description: &'a str
}

impl Finisher<'_> {
    pub const fn new(time: u8, cp: u16, durability: u8, progress: u16, uses_trained_perfection: bool, heart_and_soul: bool, description: &str) -> Finisher {
        Finisher {
            time,
            cp,
            durability,
            progress,
            heart_and_soul,
            uses_trained_perfection,
            description
        }
    }

    pub fn beats(&self, other: &Self) -> bool {
        self.cp <= other.cp && self.durability <= other.durability && self.time <= other.time && !(self.heart_and_soul && !other.heart_and_soul)
    }
}

pub const FINISHERS: [&Finisher; 21] = [
    &Finisher::new(3, 0, 1, 12, false, false, "b"),
    &Finisher::new(3, 7, 1, 18, false, false, "c"),
    // &Finisher::new(5, 12, 1, 20, false, false, "f"),
    &Finisher::new(5, 6, 1, 40, false, true, "i"),
    &Finisher::new(5, 25, 1, 27, false, false, "vc"),
    // &Finisher::new(7, 30, 1, 30, false, false, "vf"),
    &Finisher::new(7, 24, 1, 60, false, true, "vi"),
    &Finisher::new(8, 18, 1, 48, true, false, "*gb"),
    &Finisher::new(8, 25, 1, 54, true, false, "*gc"),
    // &Finisher::new(10, 30, 1, 56, true, false, "*gf"),
    &Finisher::new(10, 24, 1, 76, true, true, "*gi"),
    &Finisher::new(10, 43, 1, 81, true, false, "v*gc"),
    // &Finisher::new(12, 48, 1, 84, true, false, "v*gf"),
    &Finisher::new(12, 42, 1, 114, true, true, "v*gi"),
    &Finisher::new(6, 18, 2, 30, false, false, "pb"),
    &Finisher::new(6, 25, 2, 36, false, false, "pc"),
    // &Finisher::new(8, 30, 2, 38, false, false, "pf"),
    &Finisher::new(8, 24, 2, 58, false, true, "pi"),
    &Finisher::new(8, 43, 2, 54, false, false, "vpc"),
    // &Finisher::new(10, 48, 2, 57, false, false, "vpf"),
    &Finisher::new(8, 36, 2, 45, false, false, "vpb"),
    &Finisher::new(10, 42, 2, 87, false, true, "vpi"),
    // &Finisher::new(8, 12, 3, 32, false, false, "bf"),
    &Finisher::new(6, 7, 3, 30, false, false, "bc"),
    // &Finisher::new(8, 19, 3, 38, false, false, "cf"),
    &Finisher::new(6, 0, 3, 24, false, false, "bb"),
    &Finisher::new(6, 14, 3, 36, false, false, "cc"),
    // &Finisher::new(10, 24, 3, 40, false, false, "ff"),
    &Finisher::new(8, 6, 3, 52, false, true, "bi"),
    &Finisher::new(8, 13, 3, 58, false, true, "ci"),
    // &Finisher::new(10, 17, 3, 60, false, true, "fi"),
];

pub const OPENERS: [&str; 48] = [ 
    "Mmv1g",
    "Mmv2g",
    "Mmv1gg",
    "Mmv2gg",
    "Mmv1ggg",
    "Mmv2ggg",
    "Mmv1ig",
    "Mmv2ig",
    "Mmv1igg",
    "Mmv2igg",
    "Mmv1iggg",
    "Mmv2iggg",
    "Mmv1igg*g",
    "Mmv2igg*g",
    "Mmvi1g",
    "Mmvi2g",
    "Mmvi1gg",
    "Mmvi2gg",
    "Mmvi1ggg",
    "Mmvi2ggg",
    "Rmv1gg",
    "Rmv2gg",
    "Rmv1ggg",
    "Rmv2ggg",
    "Rmv1gggg",
    "Rmv2gggg",
    "Rmv2ggggg",
    "Rmv1ig",
    "Rmv2ig",
    "Rmv1igg",
    "Rmv2igg",
    "Rmv1iggg",
    "Rmv2iggg",
    "Rmv2igggg",
    "Rmvi1g",
    "Rmvi2g",
    "Rmvi1gg",
    "Rmvi2gg",
    "Rmvi1ggg",
    "Rmvi2ggg",
    "Rmvi2gggg",
    "R",
    "M",
    "Mmvipp",
    "M1g",
    "M2g",
    "Mi",
    "Mg"
];