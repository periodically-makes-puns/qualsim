pub mod qual;
pub mod prog;
use std::collections::HashSet;
use std::error;
use std::time::Instant;
use std::fs::read;
use std::fs::write;
use std::fmt;
use std::cmp;
use bincode;
use prog::Finisher;
use serde::{Serialize, Deserialize};
use std::io::BufReader;
use std::fs::File;
use serde_json;

type Materials = Vec<(u16, u8)>;

#[derive(Serialize, Deserialize)]
struct Statline {
    time: u8,
    cp: u16,
    cms: u16,
    ctrl: u16,
    rlvl: u16,
    dur: i8,
    prog: u32,
    qual: u32,
    has: bool,
    materials: Option<Materials>
}

impl Statline {
    fn load(filename: String) -> Result<Statline, Box<dyn error::Error>> {
        let f = File::open(filename);
        match f {
            Ok(res) => {
                match serde_json::from_reader(BufReader::new(res)) {
                    Ok(res) => {Ok(res)}
                    Err(err) => {Err(Box::new(err))}
                }
            },
            Err(err) => {Err(Box::new(err))}
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Bounds {
    cms: (u16, u16),
    ctrl: (u16, u16),
    cp: (u16, u16)    
}

#[derive(Serialize, Deserialize)]
struct Options {
    mode: String,
    incache: String,
    outcache: String,
    recipe_file: String,
    check_time: bool,
    bounds: Bounds
}

fn load_options() -> Options {
    let f = File::open("options.json").unwrap();
    let rdr = BufReader::new(f);
    serde_json::from_reader(rdr).unwrap()
}

const LV_90_PROG_DIV: f64 = 130.;
const LV_90_QUAL_DIV: f64 = 115.;
const LV_90_PROG_MUL: f64 = 80.;
const LV_90_QUAL_MUL: f64 = 70.;

fn convert(recipe: &Statline, pst: &prog::State, finisher: &Finisher, prog_unit: u16) -> Option<(qual::State, bool)> {
    // Converts a prog state to a qual state if possible. If recipe would fail, returns None
    //assert!(pst.prog as u32 * (prog_unit as u32) < recipe.prog * 10, "Opener should not finish craft");
    if (pst.prog as u32 + finisher.prog as u32) * (prog_unit as u32) < recipe.prog * 10 {
        // Check that finisher finishes craft
        return None
    }
    // check that there are resources remaining
    if recipe.cp < pst.cp + finisher.cp || 
        recipe.dur < pst.dur + finisher.dur || 
        recipe.time < pst.time + finisher.time || 
        (!recipe.has && (pst.has || finisher.has)) ||
        (pst.has && finisher.has) {
        return None
    }

    Some((qual::State {
        time: recipe.time - pst.time - finisher.time,
        cp: pst.cp - finisher.cp,
        iq: pst.iq,
        dur: pst.dur - finisher.dur,
        manip: pst.manip,
        wn: pst.wn,
        inno: 0,
        gs: 0,
        has: recipe.has && !pst.has && !finisher.has
    }, pst.reflect))
}

pub struct Rotation<'a> {
    opener: &'a str,
    extra: char,
    finisher: &'a Finisher<'a>
}

struct SimResult<'a> {
    best_qual: u32,
    best_time: u8,
    best_rot: Rotation<'a>,
    best_qst: qual::State
}

fn check_recipe<'a>(cache: &mut qual::DPCache, recipe: &mut Statline) -> SimResult<'a> {
    let prog_unit: u16 = ((recipe.cms as f64 * 10. / LV_90_PROG_DIV + 2.) * if recipe.rlvl >= 580 {LV_90_PROG_MUL} else {100.} / 100.).floor() as u16;
    let qual_unit: u16 = ((recipe.ctrl as f64 * 10. / LV_90_QUAL_DIV + 35.) * if recipe.rlvl >= 580 {LV_90_QUAL_MUL} else {100.} / 100.).floor() as u16;
    println!("Prog/100: {}", prog_unit);
    println!("Qual/100: {}", qual_unit);
    let mut min = 0;
    let mut t = (recipe.time + min) / 2;
    let mut max = recipe.time;
    
    let mut best_qual = 0;
    let mut best_rot: Option<Rotation> = None;
    let mut best_qst: Option<qual::State> = None;
    while min < max {
        dbg!(min, t, max);
        recipe.time = t;
        best_qual = 0;
        best_rot = None;
        best_qst = None;
        for opener in prog::OPENERS {
            for extra in " bcf".chars() {
                let mut st = prog::State {
                    time: 0,
                    iq: 0,
                    cp: recipe.cp,
                    dur: recipe.dur / 5,
                    manip: 0,
                    wn: 0,
                    ven: 0,
                    mm: 0,
                    has: recipe.has,
                    reflect: false,
                    prog: 0
                };
                st.apply_opener(opener, extra);
                if st.prog as u32 * prog_unit as u32 >= recipe.prog * 10 {
                    continue;
                }
                let good_finishers: Vec<&&Finisher> = prog::FINISHERS.iter().filter(|f| 
                    (f.prog + st.prog) as u32 * (prog_unit as u32) >= recipe.prog * 10).collect();
                'outer: for finisher in prog::FINISHERS {
                    for fin2 in &good_finishers {
                        if fin2.beats(finisher) && **fin2 != finisher {
                            continue 'outer;
                        }
                    }
                    let st = st.clone();
                    let res = convert(&recipe, &st, finisher, prog_unit);
                    let qst: qual::State;
                    let bonus_qual;
                    match res {
                        Some((st, reflect)) => {qst = st; bonus_qual = if reflect {qual::UNIT} else {0};}
                        None => continue
                    }
                    dbg!(format!("{}{} {}", opener, extra, finisher.desc));
                    dbg!((st.prog as u32 + finisher.prog as u32) * prog_unit as u32);
                    let (q, _method, _next) = qual::unpack_method(cache.query(&qst));
                    let q = (q + bonus_qual) as u32 * qual_unit as u32 / qual::UNIT as u32;
                    if q > best_qual {
                        best_qual = q;
                        best_qst = Some(qst);
                        best_rot = Some(Rotation {
                            opener,
                            extra,
                            finisher: &finisher
                        });
                    }
                }
            }
        }
        if best_qual >= recipe.qual {
            max = t;
        } else {
            if min == t && max < recipe.time {
                max += 1;
            }
            min = t + 1;
        }
        t = (max + min) / 2;
    }
    SimResult {
        best_qual,
        best_time: t,
        best_rot: best_rot.unwrap(),
        best_qst: best_qst.unwrap()
    }
}

fn convert_char(c: char) -> (&'static str, i32) {
    match c {
        'M' => ("Muscle Memory", 3),
        'R' => ("Reflect", 3),
        'm' => ("Manipulation", 2),
        'v' => ("Veneration", 2),
        '1' => ("Waste Not", 2),
        '2' => ("Waste Not II", 2),
        'b' => ("Basic Synthesis", 3),
        'c' => ("Careful Synthesis", 3),
        'f' => ("Observe", 3),
        'g' => ("Groundwork", 3),
        'i' => ("Heart and Soul", 3),
        _ => ("", 1)
    }
}

fn print_char(c: char) {
    let (name, wait) = convert_char(c);
    println!("/ac '{}' <wait.{}>", name, wait);
    if c == 'f' {
        println!("/ac 'Focused Synthesis' <wait.3>");
    } else if c == 'i' {
        println!("/ac 'Intensive Synthesis' <wait.3>");
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Solution {
    cms: u16,
    ctrl: u16,
    cp: u16, 
    has: bool
}

impl Solution {
    fn beats(&self, other: &Self) -> bool {
        self.cms <= other.cms && self.ctrl <= other.ctrl && self.cp <= other.cp && *self != *other && !(self.has && !other.has)
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{}/{}", self.cms, self.ctrl, self.cp, self.has)
    }
}

fn main() {
    let mut options = load_options();
    let mut cache: qual::DPCache;

    let start = Instant::now();
    if options.incache.len() > 0 {
        cache = bincode::deserialize(&read(options.incache).unwrap()).unwrap();
    } else {
        cache = qual::DPCache::new(options.check_time);
    }
    println!("Cache loaded");
    let recipe = Statline::load(options.recipe_file);
    let mut recipe = match recipe {
        Ok(res) => {res}
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };
    if options.mode == "recipe" {
        let result = check_recipe(&mut cache, &mut recipe);
        let SimResult {best_rot, best_qst, best_qual, best_time} = result;
        let finisher = format!("{}", best_rot.finisher.desc);
        for c in best_rot.opener.chars() {
            print_char(c);
        }
        if best_rot.extra != ' ' {
            print_char(best_rot.extra);
        }
        cache.print_macro(&best_qst);
        for c in finisher.chars() {
            print_char(c);
        }
        println!("Best time: {}", best_time);
        println!("Quality: {}", best_qual);
        cache.print_backtrace(&best_qst);
        println!("hits: {}", cache.hits);
        println!("items: {}", cache.items);
        println!("{}ms", start.elapsed().as_millis());
    } else if options.mode == "gearset" {
        if recipe.has { // Raise upper bound to allow specialist
            options.bounds.cms.1 += 20;
            options.bounds.ctrl.1 += 20;
            options.bounds.cp.1 += 15;
        }
        let min_prog_unit: u16 = ((options.bounds.cms.0 as f64 * 10. / LV_90_PROG_DIV + 2.) * if recipe.rlvl >= 580 {LV_90_PROG_MUL} else {100.} / 100.).floor() as u16;
        let max_prog_unit: u16 = ((options.bounds.cms.1 as f64 * 10. / LV_90_PROG_DIV + 2.) * if recipe.rlvl >= 580 {LV_90_PROG_MUL} else {100.} / 100.).floor() as u16;
        let min_qual_unit: u16 = ((options.bounds.ctrl.0 as f64 * 10. / LV_90_QUAL_DIV + 35.) * if recipe.rlvl >= 580 {LV_90_QUAL_MUL} else {100.} / 100.).floor() as u16;
        //let max_qual_unit: u16 = ((options.bounds.ctrl.1 as f64 * 10. / LV_90_QUAL_DIV + 35.) * if recipe.rlvl >= 580 {LV_90_QUAL_MUL} else {100.} / 100.).floor() as u16;
        dbg!(min_prog_unit, min_qual_unit);
        let mut solutions: HashSet<Solution> = HashSet::new();
        for target_cp in options.bounds.cp.0..=options.bounds.cp.1 {
            for opener in prog::OPENERS {
                for extra in " bcf".chars() {
                    for has in 0..=recipe.has as u8 {
                        let mut st = prog::State {
                            time: 0,
                            iq: 0,
                            cp: target_cp,
                            dur: recipe.dur / 5,
                            manip: 0,
                            wn: 0,
                            ven: 0,
                            mm: 0,
                            has: false,
                            reflect: false,
                            prog: 0
                        };
                        st.apply_opener(opener, extra);
                        if st.prog as u32 * min_prog_unit as u32 >= recipe.prog * 10 {
                            continue;
                        }
                        let opener_prog = st.prog;
                        let good_finishers: Vec<&&Finisher> = prog::FINISHERS.iter().filter(|f| 
                            (f.prog + st.prog) as u32 * (max_prog_unit as u32) >= recipe.prog * 10).collect();
                        'finLoop: for finisher in good_finishers {
                            let st = st.clone();
                            let res = convert(&recipe, &st, finisher, max_prog_unit);
                            let mut qst: qual::State;
                            let bonus_qual;
                            match res {
                                Some((st, reflect)) => {qst = st; bonus_qual = if reflect {qual::UNIT} else {0};}
                                None => continue
                            }
                            if recipe.has && has == 0 { // Special check to handle recipe HaS being weird
                                if qst.has {
                                    qst.has = false;
                                } else {
                                    continue;
                                }
                            }
                            //dbg!(format!("{}{} {}", opener, extra, finisher.desc));
                            let (q, _method, _next) = qual::unpack_method(cache.query(&qst));
                            let q = (q + bonus_qual) as f64 / qual::UNIT as f64;
                            let p = (finisher.prog + opener_prog) as f64 / 10.;
                            let min_cms: u16 = (13. * ((recipe.prog as f64 / p).ceil() * 1.25 - 2.)).ceil() as u16;
                            let min_ctrl: u16 = (11.5 * ((recipe.qual as f64 / q).ceil() * 10. / 7. - 35.)).ceil() as u16;
                            //dbg!(min_cms);
                            if min_cms > options.bounds.cms.1 || min_ctrl > options.bounds.ctrl.1 {
                                continue;
                            }
                            let pu = (recipe.prog as f64 / p as f64).ceil();
                            let qu = (recipe.qual as f64 / q as f64).ceil();
                            if pu + 2. < min_prog_unit as f64 || qu + 2. < min_qual_unit as f64{
                                //dbg!(pu, qu);
                                continue;
                            }
                            let new_sol = Solution  {
                                cms: cmp::max(min_cms, options.bounds.cms.0), 
                                ctrl: cmp::max(min_ctrl, options.bounds.ctrl.0),
                                cp: target_cp,
                                has: (has > 0) && !cache.check_endstate(&qst).has
                            };
                            solutions.retain(|sol| {
                                !new_sol.beats(sol)
                            });
                            for sol in &solutions {
                                if sol.beats(&new_sol) {
                                    continue 'finLoop;
                                }
                            }
                            //dbg!(cache.check_endstate(&qst));
                            //println!("{}", &new_sol);
                            solutions.insert(new_sol);
                        }
                    }
                }
            }
        }
        for sol in &solutions {
            println!("{}", sol);
        }
    }
    if options.outcache.len() > 0 {
        write(options.outcache, bincode::serialize(&cache).unwrap()).expect("Failed to export cache");
    }
}
