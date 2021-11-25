use crate::instant::MistInstant;
use crate::run::Run;

pub struct RunState {
    run: Run,
    timer: MistInstant,
    timer_state: TimerState,
    active_run_times: Vec<u128>,
    active_run_diffs: Vec<i128>,
    before_pause: u128,
    before_pause_split: u128,
    split: u128,
    start: u128,
    reset: bool,
    current_split: usize,
}

#[derive(PartialEq)]
pub enum TimerState {
    Running,
    NotRunning,
    Paused,
    Offset,
    Finished,
}

pub enum StateChangeRequest {
    Pause,
    Unpause,
    Split,
    Reset,
}

// commented items will be used for plugins later
pub enum StateChange {
    None,
    EnterOffset, /*{amt: u128}*/
    ExitOffset,
    EnterSplit {
        idx: usize, /*name: String, pb: u128, gold: u128 */
    },
    ExitSplit {
        idx: usize,
        /*name: String,*/ status: SplitStatus,
        time: u128,
    },
    Pause,
    Unpause,
    Finish,
    Reset {
        offset: Option<u128>,
    },
}

pub struct RunUpdate {
    change: Vec<StateChange>,
    time: u128,
    status: SplitStatus,
}

pub enum SplitStatus {
    Ahead,
    Gold,
    Behind,
    Gaining,
    Losing,
}

impl RunState {
    pub fn new(run: Run) -> Self {
        Self {
            run,
            timer: MistInstant::now(),
            timer_state: TimerState::NotRunning,
            active_run_times: vec![],
            active_run_diffs: vec![],
            before_pause: 0,
            before_pause_split: 0,
            split: 0,
            start: 0,
            reset: false,
            current_split: 0,
        }
    }
    pub fn update(&mut self, rq: StateChangeRequest) -> RunUpdate {
        // TODO logic for checking offset stuff
        let elapsed = self.timer.elapsed().as_millis();
        RunUpdate {
            change: self.handle_scrq(rq, elapsed),
            time: (elapsed - self.start) + self.before_pause,
            status: self.calc_status(),
        }
    }
    fn calc_status(&mut self) -> SplitStatus {
        // TODO
        SplitStatus::Ahead
    }
    fn handle_scrq(&mut self, rq: StateChangeRequest, elapsed: u128) -> Vec<StateChange> {
        use StateChangeRequest::*;
        match rq {
            Pause
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Offset =>
            {
                self.timer_state = TimerState::Paused;
                self.before_pause = (elapsed - self.start) + self.before_pause;
                self.before_pause_split = (elapsed - self.split) + self.before_pause_split;
            }
            Unpause if self.timer_state == TimerState::Paused => {
                self.timer_state = TimerState::Running;
                self.start = elapsed;
                self.split = elapsed;
            }
            Split if self.timer_state == TimerState::Running => {
                // TODO run updates/save file updates etc
                if self.current_split == self.run.pb_times().len() - 1 {
                    self.timer_state = TimerState::Finished;
                } else {
                    self.current_split += 1;
                    self.active_run_times.push((elapsed - self.split) + self.before_pause_split);
                }
            }
            Split if self.timer_state == TimerState::NotRunning => {
				self.timer_state = TimerState::Running;
            }
            Reset => {
                self.before_pause = 0;
                self.before_pause_split = 0;
                self.split = 0;
                self.start = 0;
                self.reset = false;
                self.active_run_diffs = vec![];
                self.active_run_times = vec![];
                self.current_split = 0;
                self.timer_state = TimerState::NotRunning;
            }
            _ => {}
        }
        vec![StateChange::None]
    }
}
