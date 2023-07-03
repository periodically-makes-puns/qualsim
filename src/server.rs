use qual::DPCache;
use scc::TreeIndex;

struct AsyncCache {
    cache: TreeIndex<u64, u64>,
    check_time: bool,
    max_dur: i8
}

