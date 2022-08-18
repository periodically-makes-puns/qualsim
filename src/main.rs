pub mod qual;
pub mod prog;
use std::time::Instant;
use std::fs::read;
use std::fs::write;
use bincode;
use prog::Finisher;
use std::env;
use serde::{Serialize, Deserialize};
use std::io::BufReader;
use std::fs::File;
use serde_json;

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
    has: bool
}

fn load_recipe() -> Statline {
    let f = File::open("recipe.json").unwrap();
    let rdr = BufReader::new(f);
    serde_json::from_reader(rdr).unwrap()
}

const LV_90_PROG_DIV: f64 = 130.;
const LV_90_QUAL_DIV: f64 = 115.;
const LV_90_PROG_MUL: f64 = 80.;
const LV_90_QUAL_MUL: f64 = 70.;

fn convert(recipe: &Statline, pst: &prog::State, finisher: &Finisher, prog_unit: u16) -> Option<(qual::State, bool)> {
    // Converts a prog state to a qual state if possible. If recipe would fail, returns None
    assert!(pst.prog as u32 * (prog_unit as u32) < recipe.prog * 10, "Opener should not finish craft");
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

struct Rotation<'a> {
    opener: &'a str,
    extra: char,
    finisher: &'a Finisher<'a>
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let input = &args[1];
    let output = &args[2];
    let mut cache: qual::DPCache;

    let start = Instant::now();
    if input.len() > 1 {
        cache = bincode::deserialize(&read(input).unwrap()).unwrap();
    } else {
        cache = qual::DPCache::new();
    }
    println!("Cache loaded");

    let mut recipe = load_recipe();
    println!("Recipe loaded");
    let prog_unit: u16 = ((recipe.cms as f64 * 10. / LV_90_PROG_DIV + 2.) * if recipe.rlvl > 580 {LV_90_PROG_MUL} else {100.} / 100.).floor() as u16;
    let qual_unit: u16 = ((recipe.ctrl as f64 * 10. / LV_90_QUAL_DIV + 35.) * if recipe.rlvl > 580 {LV_90_QUAL_MUL} else {100.} / 100.).floor() as u16;
    println!("Prog/100: {}", prog_unit);
    println!("Qual/100: {}", qual_unit);
    let mut min = 0;
    let mut t = 45;
    let mut max = 90;
    
    let mut best_qual;
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
            min = t + 1;
        }
        t = (max + min) / 2;
    }
    println!("Best time: {}", t);
    if let Some(rot) = best_rot {
        println!("{}{} {}", rot.opener, rot.extra, rot.finisher.desc);
        cache.print_backtrace(&best_qst.unwrap());
    }
    println!("hits: {}", cache.hits);
    println!("items: {}", cache.items);
    println!("{}ms", start.elapsed().as_millis());
    if output.len() > 1 {
        write(output, bincode::serialize(&cache).unwrap())
    } else {
        write("res.txt", b"success")
    }
}
