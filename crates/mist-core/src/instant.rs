#![allow(non_camel_case_types)]
pub use inner::MistInstant;

#[cfg(unix)]
mod inner {
    pub use innerinner::MistInstant;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    mod innerinner {
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct mach_timebase_info {
            numer: u32,
            denom: u32,
        }
        fn info() -> mach_timebase_info {
            static INFO_BITS: AtomicU64 = std::sync::atomic::AtomicU64::new(0);

            let info_bits = INFO_BITS.load(std::cmp::Ordering::Relaxed);
            if info_bits != 0 {
                return info_from_bits(info_bits);
            }

            extern "C" {
                fn mach_timebase_info(info: *mut mach_timebase_info) -> libc::c_int;
            }

            let mut info = info_from_bits(0);
            unsafe {
                mach_timebase_info(&mut info);
            }
            INFO_BITS.store(info_to_bits(info), std::cmp::Ordering::Relaxed);
            info
        }

        #[inline]
        fn info_to_bits(info: mach_timebase_info) -> u64 {
            ((info.denom as u64) << 32) | (info.numer as u64)
        }

        #[inline]
        fn info_from_bits(bits: u64) -> mach_timebase_info {
            mach_timebase_info {
                numer: bits as u32,
                denom: (bits >> 32) as u32,
            }
        }
        #[derive(Copy, Clone)]
        pub struct MistInstant {
            t: u64,
        }

        impl MistInstant {
            pub fn now() -> Self {
                extern "C" {
                    fn mach_continuous_time() -> u64;
                }
                Self {
                    t: unsafe { mach_continuous_time() },
                }
            }
        }

        impl std::ops::Sub<MistInstant> for MistInstant {
            type Output = std::time::Duration;
            fn sub(self, other: MistInstant) -> Self::Output {
                let diff = self.t
                    .checked_sub(other.t)
                    .expect("overflow when subtracting instants");
                let info = info();
                let nanos = ((diff / info.denom) * numer) + (((diff % info.denom) * numer) / info.denom);
                std::time::Duration::new(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32)
            }
        }
    }

    #[cfg(target_os = "linux")]
    mod innerinner {
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

        impl MistInstant {
            pub fn now() -> Self {
                Self {
                    t: now(libc::CLOCK_BOOTTIME),
                }
            }
        }

        #[cfg(not(any(target_os = "dragonfly", target_os = "espidf")))]
        pub type clock_t = libc::c_int;
        #[cfg(any(target_os = "dragonfly", target_os = "espidf"))]
        pub type clock_t = libc::c_ulong;

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
