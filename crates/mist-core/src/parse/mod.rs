//! Parse [`Runs`](crate::timer::Run) from their file representations.
mod msf;
pub use msf::MsfParser;

#[cfg(feature = "lss")]
mod lss;
#[cfg(feature = "lss")]
pub use lss::LssParser;

use crate::timer::Run;

pub(crate) fn sanify_run(run: &Run) -> Run {
    let mut run = run.clone();
    let len = run.splits().len();
    let mut golds = run.gold_times().to_owned();
    let mut times = run.pb_times().to_owned();
    let mut sums = run.sum_times().to_owned();
    if golds.len() != len {
        golds.resize_with(len, Default::default);
        run.set_gold_times(&golds);
    }
    if times.len() != len {
        times.resize_with(len, Default::default);
        run.set_pb_times(&times);
    }
    if sums.len() != len {
        sums.resize_with(len, Default::default);
        run.set_sum_times(&sums);
    }
    run
}
