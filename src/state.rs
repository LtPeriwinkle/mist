//state of timer, might implement real state switching eventually
#[derive(Debug)]
pub enum TimerState {
    OffsetCountdown {
        amt: u128,
    },
    Running {
        timestamp: u128,
    },
    Paused {
        time: u128,
        split: u128,
        time_str: String,
    },
    NotRunning {
        time_str: String,
    },
}
