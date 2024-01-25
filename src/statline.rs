

pub struct CrafterStats {
    lvl: u8,
    cp: u16,
    cms: u16,
    ctrl: u16
}

pub struct Recipe {
    rlvl: u16,
    prog: u32,
    qual: u32,
    dur: u8,
    pdiv: u16,
    qdiv: u16,
    pmod: u16,
    qmod: u16,
    reqqual: Option<u32>
}

pub struct CombinedCraftInfo {
    prog: u32,
    qual: u32,
    dur: u8,
    cp: u16,
    p100: u16,
    q100: u16
}

pub fn clamp(v: u8, lb: u8, ub: u8) -> u8 {
    if v < lb {
        lb
    } else if v > ub {
        ub
    } else {
        v
    }
}

pub fn combine_info(recipe: &Recipe, stats: &CrafterStats) -> CombinedCraftInfo {
    let clvl = CLVL_TABLE[clamp(stats.lvl-1, 0, 89) as usize];
    let p100num: u64 = (stats.cms as u64 * 10 + 2 * recipe.pdiv as u64) * (if clvl <= recipe.rlvl {recipe.pmod as u64} else {100});
    let p100denom: u64 = recipe.pdiv as u64 * 100;
    let p100 = (p100num / p100denom) as u16;
    let q100num: u64 = (stats.ctrl as u64 * 10 + 35 * recipe.qdiv as u64) * (if clvl <= recipe.rlvl {recipe.qmod as u64} else {100});
    let q100denom: u64 = recipe.qdiv as u64 * 100;
    let q100 = (q100num / q100denom) as u16;
    
    CombinedCraftInfo {
        prog: recipe.prog,
        qual: recipe.reqqual.unwrap_or(recipe.qual),
        dur: recipe.dur,
        cp: stats.cp,
        p100,
        q100
    }
}


pub const CLVL_TABLE: [u16; 90] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 
    11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 
    31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 
    120, 125, 130, 133, 136, 139, 142, 145, 148, 150, 
    260, 265, 270, 273, 276, 279, 282, 285, 288, 290, 
    390, 395, 400, 403, 406, 409, 412, 415, 418, 420, 
    517, 520, 525, 530, 535, 540, 545, 550, 555, 560
];