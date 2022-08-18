#[derive(Debug, Clone)]
pub struct State {
    pub time: u8, // 0-89, 7 bits
    pub iq: u8, // 0-10, 4 bits
    pub cp: u16, // 0-699, 10 bits
    pub dur: i8, // 0-16, 5 bits
    pub manip: i8, // 0-8, 4 bits
    pub wn: i8, // 0-8, 4 bits
    pub ven: i8,
    pub mm: i8, 
    pub has: bool,
    pub reflect: bool,
    pub prog: u16
}



mod actions {

    #[derive(PartialEq)]
    pub enum Status {
        NONE,
        MANIP,
        WN,
        VEN,
        MM
    }
    pub struct Action {
        pub prog: u16, // Efficiency x10
        pub dur: i8, // 5dur = 1
        pub cp: u16,
        pub status: Status,
        pub duration: i8
    }

    impl Action {
        pub const fn new(prog: u16, dur: i8, cp: u16, status: Status, duration: i8) -> Action {
            Action {
                prog,
                dur,
                cp,
                status,
                duration
            }
        }
    }

	pub const BASIC: Action = Action::new(12, 2, 0, Status::NONE, 0 );
	pub const CAREFUL: Action = Action::new(18, 2, 7, Status::NONE, 0 );
	pub const FOCUSED: Action = Action::new(20, 2, 12, Status::NONE, 0 );
	pub const PRUDENT: Action = Action::new(18, 1, 18, Status::NONE, 0 );
	pub const GROUNDWORK: Action = Action::new(36, 4, 18, Status::NONE, 0 );
	pub const MUMEN: Action = Action::new(30, 2, 6, Status::MM, 5 );
	pub const VENER: Action = Action::new(0, 0, 18, Status::VEN, 4 );
	pub const MANIPULATION: Action = Action::new(0, 0, 96, Status::MANIP, 8 );
	pub const WN1: Action = Action::new(0, 0, 56, Status::WN, 4 );
	pub const WN2: Action = Action::new(0, 0, 98, Status::WN, 8 );
	pub const INTENSIVE: Action = Action::new(40, 2, 6, Status::NONE, 0 );
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
            self.dur -= 2;
            self.cp -= 18;
            self.iq += 2;
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
            _ => {panic!("Bad action char");}
        });
    }

    pub fn apply_action(&mut self, act: &actions::Action) {
        let w = self.wn;
	    let v = self.ven;
	    let m = self.manip;
	    let d = self.dur;
        if act.cp == 12 {
            self.tick_statuses(true);
        }
        if (act.dur > self.dur * (if self.wn > 0 {2} else {1})) || act.cp > self.cp || (act.dur == 1 && self.wn == 0) {
            self.wn = w;
            self.ven = v;
            self.manip = m;
            self.dur = d;
            return;
        }
        self.dur -= act.dur >> (if self.wn > 0 {1} else {0});
	    self.cp -= act.cp;
	    let mut p = act.prog;
        if p == 40 {self.has = true;}
        if self.ven > 0 {p += act.prog / 2;}
        if self.mm > 0 && p > 0 {
            p += act.prog;
            self.mm = 0;
        }
        if p > 0 {
            if act.prog == 20 {self.time += 2;}
            self.time += 3;
        } else {
            self.time += 2;
        }
        self.prog += p;
        let tick_manip = act.status != actions::Status::MANIP;
        self.tick_statuses(tick_manip);
        match act.status {
            actions::Status::MANIP => {self.manip = 8;}
            actions::Status::WN => {self.wn = act.duration;}
            actions::Status::VEN => {self.ven = 4;}
            actions::Status::MM => {self.mm = 5;}
            _ => {}
        }
    }

    pub fn tick_statuses(&mut self, tick_manip: bool) {
        if self.wn > 0 {self.wn -= 1;}
        if self.ven > 0 {self.ven -= 1;}
        if self.mm > 0 {self.mm -= 1;}
        if self.manip > 0 && tick_manip {
            self.manip -= 1;
            self.dur += 1;
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Finisher<'a> {
    pub time: u8,
    pub cp: u16,
    pub dur: i8,
    pub prog: u16,
    pub has: bool,
    pub desc: &'a str
}

impl Finisher<'_> {
    pub const fn new(time: u8, cp: u16, dur: i8, prog: u16, has: bool, desc: &str) -> Finisher {
        Finisher {
            time,
            cp,
            dur,
            prog,
            has,
            desc
        }
    }

    pub fn beats(&self, other: &Self) -> bool {
        self.cp <= other.cp && self.dur <= other.dur && self.time <= other.time && !(self.has && !other.has)
    }
}

pub const FINISHERS: [&Finisher; 24] = [
    &Finisher::new(3, 0, 1, 12, false, "b"),
    &Finisher::new(3, 7, 1, 18, false, "c"),
    &Finisher::new(5, 12, 1, 20, false, "f"),
    &Finisher::new(5, 6, 1, 40, true, "i"),
    &Finisher::new(5, 25, 1, 27, false, "vc"),
    &Finisher::new(7, 30, 1, 30, false, "vf"),
    &Finisher::new(7, 24, 1, 60, true, "vi"),
    &Finisher::new(6, 18, 2, 30, false, "pb"),
    &Finisher::new(6, 25, 2, 36, false, "pc"),
    &Finisher::new(8, 30, 2, 38, false, "pf"),
    &Finisher::new(8, 24, 2, 58, true, "pi"),
    &Finisher::new(8, 43, 2, 54, false, "vpc"),
    &Finisher::new(10, 48, 2, 57, false, "vpf"),
    &Finisher::new(8, 36, 2, 45, false, "vpb"),
    &Finisher::new(10, 42, 2, 87, true, "vpi"),
    &Finisher::new(8, 12, 3, 32, false, "bf"),
    &Finisher::new(6, 7, 3, 30, false, "bc"),
    &Finisher::new(8, 19, 3, 38, false, "cf"),
    &Finisher::new(6, 0, 3, 24, false, "bb"),
    &Finisher::new(6, 14, 3, 36, false, "cc"),
    &Finisher::new(10, 24, 3, 40, false, "ff"),
    &Finisher::new(8, 6, 3, 52, true, "bi"),
    &Finisher::new(8, 13, 3, 58, true, "ci"),
    &Finisher::new(10, 17, 3, 60, true, "fi"),
];

pub const OPENERS: [&str; 39] = [ 
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
];