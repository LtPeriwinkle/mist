//! Internal state of mist.

use super::format;
use super::Comparison as Comp;
use super::MistInstant;
use super::Run;
use super::{DiffType, TimeType};
use std::cell::RefCell;
use std::rc::Rc;

/// The state of the loaded run.
///
/// This holds everything needed for timekeeping, updating the state of the run (e.g. paused/running),
/// keeping comparisons, and storing times.
pub struct RunState {
    run: Rc<RefCell<Run>>,
    timer: MistInstant,
    timer_state: TimerState,
    run_status: SplitStatus,
    comparison: Comp,
    run_times: Vec<TimeType>,
    run_diffs: Vec<DiffType>,
    run_golds: Vec<bool>,
    sum_comp_times: Vec<u128>,
    before_pause: u128,
    before_pause_split: u128,
    split: u128,
    start: u128,
    time: u128,
    current_split: usize,
    needs_save: bool,
    set_times: bool,
}

#[derive(PartialEq, Debug)]
enum TimerState {
    Running,
    NotRunning,
    Paused,
    Offset,
    Finished,
}

/// A request to the [`RunState`] to change its state.
#[derive(Debug)]
pub enum StateChangeRequest {
    /// No request.
    None,
    /// Pause the timer.
    Pause,
    /// Start the timer, advance a split, or stop the timer.
    Split,
    /// Undo a split, returning to the previous one.
    Unsplit,
    /// Skip a split, leaving it with no time.
    Skip,
    /// Set the timer back to the beginning.
    Reset,
    /// Change the comparison. `true` for next, `false` for prev.
    Comparison(bool),
}

// commented items will be used for plugins later
/// A single change in state.
#[derive(Debug)]
pub enum StateChange {
    /// No change.
    None,
    /// Start the timer in an offset.
    EnterOffset, /*{amt: u128}*/
    /// Start timing forwards after an offset.
    ExitOffset,
    /// Start a split.
    EnterSplit {
        /// Index of split that was entered.
        idx: usize, /*name: String, pb: u128, gold: u128 */
    },
    /// Finish a split.
    ExitSplit {
        /// Index of split that was finished.
        idx: usize,
        /*name: String,*/ status: SplitStatus,
        /// Amount of time spent on split.
        time: u128,
        /// Difference from comparison.
        diff: i128,
    },
    Pause,
    Unpause {
        status: SplitStatus,
    },
    /// Exited the last split, so the run is over.
    Finish,
    Reset {
        offset: Option<u128>,
    },
    ComparisonChanged {
        comp: Comp,
    },
}

/// Update returned from the [`RunState`].
///
/// One of these is returned every frame when [`update`](RunState::update) is called.
/// Since there could be multiple requests in one frame, and each request can cause
/// multiple state changes (e.g. [`ExitSplit`](StateChange::ExitSplit) and [`EnterSplit`](StateChange::EnterSplit)),
/// a [`Vec`] is used to store the state changes.
pub struct RunUpdate {
    /// All changes in state that occurred during a call to [`update`](RunState::update).
    pub change: Vec<StateChange>,
    /// Amount of time that has passed in the current split.
    pub split_time: u128,
    /// Amount of time that has passed in the run as a whole.
    pub time: u128,
    /// Whether the run is in an offset or not.
    pub offset: bool,
    /// Whether run is ahead, behind, etc.
    pub status: SplitStatus,
}

/// Status of an active run.
///
/// Usually corresponds to colors in the renderer.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SplitStatus {
    /// Timer is not running or no comparison.
    None,
    /// Runner is faster than comparison.
    Ahead,
    /// Runner is behind comparison but could save time.
    Gaining,
    /// Runner completed a split at their fastest time.
    Gold,
    /// Runner is slower than comparison.
    Behind,
    /// Runner is ahead of comparison, but is too slow to maintain previous pace.
    Losing,
}

impl RunState {
    /// Create a new [`RunState`].
    pub fn new(run: Rc<RefCell<Run>>) -> Self {
        let sum_comp_times = format::split_time_sum(&run.borrow().pb_times_u128());
        let len = run.borrow().pb_times().len();
        Self {
            run,
            timer: MistInstant::now(),
            timer_state: TimerState::NotRunning,
            comparison: Comp::PersonalBest,
            run_status: SplitStatus::None,
            run_times: vec![TimeType::Time(0); len],
            run_diffs: vec![DiffType::Time(0); len],
            run_golds: vec![false; len],
            sum_comp_times,
            before_pause: 0,
            before_pause_split: 0,
            split: 0,
            start: 0,
            time: 0,
            current_split: 0,
            needs_save: false,
            set_times: false,
        }
    }

    /// Update the [`RunState`].
    ///
    /// Because multiple inputs can occur in the same frame, multiple state changes can be requested.
    /// This handles all of them at once, and returns a [`RunUpdate`]. Also handles updating timing,
    /// setting times to a [`Run`] once they are solidified, running offsets, etc. Prevents
    /// illegal state changes from occurring by ignoring the request and returning [`StateChange::None`] in
    /// the [`RunUpdate`]
    pub fn update(&mut self, rq: &[StateChangeRequest]) -> RunUpdate {
        let elapsed = self.timer.elapsed().as_millis();
        if self.timer_state == TimerState::Running || self.timer_state == TimerState::Offset {
            self.time = (elapsed - self.start) + self.before_pause;
        }

        // have to set pb times here or else the renderer sees them too early...
        if self.set_times {
            self.run.borrow_mut().set_pb_times(&self.run_times);
            self.set_times = false;
        }

        let mut change = rq.iter().fold(Vec::new(), |mut vec, request| {
            vec.append(&mut self.handle_scrq(request, elapsed));
            vec
        });
        if self.timer_state == TimerState::Offset && self.run.borrow().offset().val() <= self.time {
            self.timer_state = TimerState::Running;
            self.start = elapsed;
            self.split = elapsed;
            change.push(StateChange::EnterSplit { idx: 0 });
        }

        self.calc_status();
        RunUpdate {
            change,
            split_time: (elapsed - self.split) + self.before_pause_split,
            time: self.time,
            offset: self.timer_state == TimerState::Offset,
            status: self.run_status,
        }
    }

    /// Whether the [`Run`] has been changed and needs to be saved.
    pub fn needs_save(&self) -> bool {
        self.needs_save
    }

    /// Whether the timer is currently running.
    pub fn is_running(&self) -> bool {
        self.timer_state == TimerState::Running
    }

    fn calc_status(&mut self) {
        if self.comparison == Comp::None || self.timer_state != TimerState::Running {
            self.run_status = SplitStatus::None;
            return;
        }
        let run = self.run.borrow();
        if run.pb_times().is_empty() || run.pb_times().len() == 1 {
            if run.pb().is_none() || self.time < run.pb().val() {
                self.run_status = SplitStatus::Ahead;
            } else {
                self.run_status = SplitStatus::Behind;
            }
        } else {
            let buffer = if self.current_split != 0 {
                self.run_diffs[self.current_split - 1].raw()
            } else {
                0
            };
            let allowed = self.sum_comp_times[self.current_split] as i128;
            if allowed == 0 {
                self.run_status = SplitStatus::Ahead;
                return;
            }
            let time = self.time as i128;
            // if the last split was ahead of comparison split
            if buffer < 0 {
                let allowed = allowed - buffer;
                // if the runner has spent more time than allowed they have to be behind
                if time > allowed {
                    self.run_status = SplitStatus::Behind;
                // if they have spent less than the time it would take to become behind but more time than they took in the pb,
                // then they are losing time but still ahead. default color for this is lightish green like LiveSplit
                } else if time < allowed && time > allowed + buffer {
                    self.run_status = SplitStatus::Losing;
                // if neither of those are true the runner must be ahead
                } else {
                    self.run_status = SplitStatus::Ahead;
                }
            // if last split was behind comparison split
            } else {
                // ahead of allowed, must be ahead
                if time < allowed {
                    self.run_status = SplitStatus::Ahead;
                // if the runner has gone over the amount of time they should take but are still on better pace than
                // last split then they are making up time. a sort of light red color like livesplit
                } else if time > allowed && time < allowed + buffer {
                    self.run_status = SplitStatus::Gaining;
                // if they are behind both the allowed time and their current pace they must be behind
                } else if time > allowed + buffer {
                    self.run_status = SplitStatus::Behind;
                }
            }
        }
    }

    fn handle_scrq(&mut self, rq: &StateChangeRequest, elapsed: u128) -> Vec<StateChange> {
        use StateChangeRequest::*;
        match rq {
            Pause
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Offset =>
            {
                self.timer_state = TimerState::Paused;
                self.before_pause = self.time;
                self.before_pause_split += elapsed - self.split;
                return vec![StateChange::Pause];
            }
            Pause if self.timer_state == TimerState::Paused => {
                self.timer_state = TimerState::Running;
                self.start = elapsed;
                self.split = elapsed;
                return vec![StateChange::Unpause {
                    status: self.run_status,
                }];
            }
            Split if self.timer_state == TimerState::Running => {
                let time = (elapsed - self.split) + self.before_pause_split;
                self.split = elapsed;
                self.before_pause_split = 0;
                self.run_times[self.current_split] = TimeType::Time(time);
                self.run_diffs[self.current_split] =
                    DiffType::Time(if self.comparison == Comp::PersonalBest {
                        time as i128
                            - self.run.borrow().pb_times()[self.current_split].val() as i128
                    } else if self.comparison == Comp::Golds {
                        time as i128
                            - self.run.borrow().gold_times()[self.current_split].val() as i128
                    } else if self.comparison == Comp::Average {
                        let sum = self.run.borrow().sum_times()[self.current_split];
                        time as i128
                            - (sum.1 / {
                                if sum.0 == 0 {
                                    1
                                } else {
                                    sum.0
                                }
                            }) as i128
                    } else {
                        0
                    });
                let mut sum = self.run.borrow().sum_times()[self.current_split];
                sum.0 += 1;
                sum.1 += time;
                self.run.borrow_mut().set_sum_time(self.current_split, sum);
                self.needs_save = true;
                if self.run.borrow().gold_times()[self.current_split].is_none()
                    || time < self.run.borrow().gold_times()[self.current_split].val()
                {
                    self.run_golds[self.current_split] = true;
                    self.run_status = SplitStatus::Gold;
                }
                let sum = format::split_time_sum(
                    &self.run_times.iter().map(|t| t.raw()).collect::<Vec<_>>(),
                )[self.current_split];
                let diff = sum as i128
                    - format::split_time_sum(&self.run.borrow().pb_times_u128())[self.current_split]
                        as i128;
                if self.current_split == self.run.borrow().pb_times().len() - 1 {
                    {
                        let mut run = self.run.borrow_mut();
                        for idx in self
                            .run_golds
                            .iter()
                            .enumerate()
                            .filter(|(_, &i)| i)
                            .map(|(idx, _)| idx)
                        {
                            run.set_gold_time(idx, self.run_times[idx]);
                        }
                    }
                    self.timer_state = TimerState::Finished;
                    if self.run.borrow().pb().is_none() || self.time < self.run.borrow().pb().val()
                    {
                        self.set_times = true;
                        self.run.borrow_mut().set_pb(TimeType::Time(self.time));
                    }
                    return vec![
                        StateChange::ExitSplit {
                            idx: self.current_split,
                            status: self.run_status,
                            time: self.run_times[self.current_split].val(),
                            diff,
                        },
                        StateChange::Finish,
                    ];
                } else {
                    self.current_split += 1;
                    return vec![
                        StateChange::ExitSplit {
                            idx: self.current_split - 1,
                            status: self.run_status,
                            time: self.run_times[self.current_split - 1].val(),
                            diff,
                        },
                        StateChange::EnterSplit {
                            idx: self.current_split,
                        },
                    ];
                }
            }
            Split if self.timer_state == TimerState::NotRunning => {
                self.start = elapsed;
                self.split = elapsed;
                self.time = 0;
                if self.run.borrow().offset().is_time() {
                    self.timer_state = TimerState::Offset;
                    return vec![StateChange::EnterOffset];
                } else {
                    self.timer_state = TimerState::Running;
                    return vec![StateChange::EnterSplit { idx: 0 }];
                }
            }
            Unsplit if self.timer_state == TimerState::Running && self.current_split != 0 => {
                self.current_split -= 1;
                self.before_pause_split = 0;
                self.split -= self.run_times[self.current_split].raw();
                self.run_diffs[self.current_split] = DiffType::Time(0);
                self.run_times[self.current_split] = TimeType::Time(0);
                self.run_golds[self.current_split] = false;
                return vec![StateChange::EnterSplit {
                    idx: self.current_split,
                }];
            }
            Reset => {
                self.before_pause = 0;
                self.before_pause_split = 0;
                self.split = 0;
                self.start = 0;
                let len = self.run.borrow().pb_times().len();
                self.run_diffs = vec![DiffType::Time(0); len];
                self.run_times = vec![TimeType::Time(0); len];
                self.run_golds = vec![false; len];
                self.current_split = 0;
                self.timer_state = TimerState::NotRunning;
                return vec![StateChange::Reset {
                    offset: self.run.borrow().offset().to_option(),
                }];
            }
            Skip if self.timer_state == TimerState::Running => {
                let time = (elapsed - self.split) + self.before_pause_split;
                self.run_times[self.current_split] = TimeType::Skipped(time);
                self.run_diffs[self.current_split] =
                    DiffType::Skipped(if self.comparison == Comp::PersonalBest {
                        time as i128
                            - self.run.borrow().pb_times()[self.current_split].val() as i128
                    } else if self.comparison == Comp::Golds {
                        time as i128
                            - self.run.borrow().gold_times()[self.current_split].val() as i128
                    } else if self.comparison == Comp::Average {
                        let sum = self.run.borrow().sum_times()[self.current_split];
                        (time
                            - (if sum.0 == 0 {
                                sum.1 / sum.0
                            } else {
                                sum.1.val()
                            })) as i128
                    } else {
                        0
                    });
                self.split = elapsed;
                self.before_pause_split = 0;
                if self.current_split == self.run.borrow().pb_times().len() - 1 {
                    self.timer_state = TimerState::Finished;
                    return vec![
                        StateChange::ExitSplit {
                            idx: self.current_split,
                            status: self.run_status,
                            time: 0,
                            diff: 0,
                        },
                        StateChange::Finish,
                    ];
                } else {
                    self.current_split += 1;
                    return vec![
                        StateChange::ExitSplit {
                            idx: self.current_split - 1,
                            status: self.run_status,
                            time: 0,
                            diff: 0,
                        },
                        StateChange::EnterSplit {
                            idx: self.current_split,
                        },
                    ];
                }
            }
            Comparison(n) => {
                if *n {
                    self.comparison.next();
                } else {
                    self.comparison.prev();
                }
                match self.comparison {
                    Comp::PersonalBest => {
                        self.sum_comp_times =
                            format::split_time_sum(&self.run.borrow().pb_times_u128());
                    }
                    Comp::Golds => {
                        self.sum_comp_times =
                            format::split_time_sum(&self.run.borrow().gold_times_u128());
                    }
                    Comp::Average => {
                        self.sum_comp_times = format::split_time_sum(
                            &self
                                .run
                                .borrow()
                                .sum_times()
                                .iter()
                                .map(|&(n, t)| if n != 0 { t / n } else { t.val() })
                                .collect::<Vec<_>>(),
                        )
                    }
                    Comp::None => {
                        self.sum_comp_times = vec![0; self.run.borrow().pb_times().len()];
                    }
                }
                return vec![StateChange::ComparisonChanged {
                    comp: self.comparison,
                }];
            }
            _ => {}
        }
        vec![StateChange::None]
    }
}
