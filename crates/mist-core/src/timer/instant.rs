#![allow(non_camel_case_types)]
pub use inner::platform::MistInstant;

mod inner {
    #[cfg(any(not(feature = "instant"), windows))]
    pub mod platform {
        /// An arbitrary point in time, used for timing speedruns.
        ///
        /// Alias to [`Instant`](std::time::Instant) for use on Windows (since its Instant already behaves how we want),
        /// or on platforms that lack the functions for continuous time.
        pub type MistInstant = std::time::Instant;
    }

    #[cfg(all(feature = "instant", unix))]
    pub mod platform {
        use std::cmp::Ordering;
        use std::time::Duration;
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
            fn partial_cmp(&self, other: &Timespec) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for Timespec {
            fn cmp(&self, other: &Timespec) -> Ordering {
                let me = (self.t.tv_sec, self.t.tv_nsec);
                let other = (other.t.tv_sec, other.t.tv_nsec);
                me.cmp(&other)
            }
        }

        impl Timespec {
            fn sub_timespec(&self, other: &Timespec) -> Option<Duration> {
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
                    Some(Duration::new(secs, nsec))
                } else {
                    None
                }
            }
        }

        #[derive(Copy, Clone)]
        /// An arbitrary point in time, used for timing speedruns.
        pub struct MistInstant {
            t: Timespec,
        }

        impl std::ops::Sub<MistInstant> for MistInstant {
            type Output = Duration;
            fn sub(self, other: MistInstant) -> Self::Output {
                self.t
                    .sub_timespec(&other.t)
                    .expect("overflow when subtracting instants")
            }
        }

        impl MistInstant {
            /// Create a [`MistInstant`] referring to the point in time that is 'now'.
            pub fn now() -> Self {
                let mut ts = Timespec {
                    t: libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                };
                #[cfg(target = "linux")]
                {
                    use std::sync::atomic::AtomicBool;
                    use std::sync::atomic::Ordering as AtomicOrd;
                    static NO_BOOTTIME: AtomicBool = AtomicBool::new(false);
                    if !NO_BOOTTIME.load(AtomicOrd::Relaxed) {
                        let r = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts.t) };
                        if r != 0 {
                            NO_BOOTTIME.store(true, AtomicOrd::Relaxed);
                        } else {
                            return Self { t: ts };
                        }
                    }
                }
                let r = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts.t) };
                if r != 0 {
                    panic!("clock_gettime is broken");
                }
                Self { t: ts }
            }
            /// Find the amount of time that has passed since this [`MistInstant`] was created.
            pub fn elapsed(&self) -> std::time::Duration {
                Self::now() - *self
            }
        }
    }
}
