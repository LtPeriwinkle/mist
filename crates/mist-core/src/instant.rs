#![allow(non_camel_case_types)]
pub use inner::MistInstant;

#[cfg(unix)]
mod inner {
    #[derive(Copy, Clone)]
    struct Timespec {
        t: libc::timespec,
    }

    impl PartialEq for Timespec {
        fn eq(&self, other: &Timespec) -> bool {
            self.t.tv_sec == other.t.tv_sec && self.t.tv_nsec == other.t.tv_nsec
        }
    }

    impl Eq for Timespec {}

    impl PartialOrd for Timespec {
        fn partial_cmp(&self, other: &Timespec) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Timespec {
        fn cmp(&self, other: &Timespec) -> std::cmp::Ordering {
            let me = (self.t.tv_sec, self.t.tv_nsec);
            let other = (other.t.tv_sec, other.t.tv_nsec);
            me.cmp(&other)
        }
    }

    impl Timespec {
        fn sub_timespec(&self, other: &Timespec) -> Option<std::time::Duration> {
            if self >= other {
                let (secs, nsec) = if self.t.tv_nsec >= other.t.tv_nsec {
                    (
                        (self.t.tv_sec - other.t.tv_sec) as u64,
                        (self.t.tv_nsec - other.t.tv_nsec) as u32,
                    )
                } else {
                    (
                        (self.t.tv_sec - other.t.tv_sec - 1) as u64,
                        self.t.tv_nsec as u32 + 1_000_000_000u32 - other.t.tv_nsec as u32,
                    )
                };
                Some(std::time::Duration::new(secs, nsec))
            } else {
                None
            }
        }
    }
    #[derive(Copy, Clone)]
    pub struct MistInstant {
        t: Timespec,
    }
    #[cfg(target_os = "linux")]
    impl MistInstant {
        pub fn now() -> Self {
            Self {
                t: now(libc::CLOCK_BOOTTIME),
            }
        }
    }
    #[cfg(target_os = "macos")]
    impl MistInstant {
        pub fn now() -> Self {
            Self {
                t: now(libc::CLOCK_MONOTONIC),
            }
        }
    }

    pub type clock_t = libc::c_int;

    fn now(clock: clock_t) -> Timespec {
        let mut t = Timespec {
            t: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        };
        let r = unsafe { libc::clock_gettime(clock, &mut t.t) };
        if r == -1 {
            panic!("couldn't clock_gettime");
        }
        t
    }

    impl std::ops::Sub<MistInstant> for MistInstant {
        type Output = std::time::Duration;
        fn sub(self, other: MistInstant) -> Self::Output {
            self.t
                .sub_timespec(&other.t)
                .expect("overflow when subtracting instants")
        }
    }
}

impl MistInstant {
    pub fn elapsed(&self) -> std::time::Duration {
        Self::now() - *self
    }
}

#[cfg(windows)]
mod inner {
    pub struct MistInstant(std::time::Instant);
    impl MistInstant {
        pub fn now() -> Self {
            MistInstant(std::time::Instant::now())
        }
        pub fn elapsed() -> std::time::Duration {
            self.0.elapsed()
        }
    }
}
