#![allow(non_camel_case_types)]
pub use inner::MistInstant;

#[cfg(unix)]
mod inner {
    pub use innerinner::MistInstant;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    mod innerinner {
        #[derive(Copy, Clone)]
        pub struct MistInstant {
            t: u64
        }
        impl MistInstant {
            pub fn now() -> Self {
                extern "C" {
                    fn mach_continuous_time() -> u64;
                }
                MistInstant {t: unsafe {mach_continuous_time()}}
            }
        }
        impl std::ops::Sub<MistInstant> for MistInstant {
            type Output = std::time::Duration;
            fn sub(self, other: MistInstant) -> Self::Output {
                self.t.checked_sub(other.t).expect("overflow when subtracting instants")
            }
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    mod innerinner {
        #[derive(Copy, Clone)]
        struct Timespec {
            t: libc::timespec
        }
        impl Timespec {
            fn sub_timespec(&self, other: &Timespec) -> Option<std::time::Duration> {
                if self.t.tv_sec >= other.t.tv_sec && self.t.tv_nsec >= other.t.tv_nsec {
                    let (sec, nsec) = ((self.t.tv_sec - other.t.tv_sec) as u64, (self.t.tv_nsec - other.t.tv_nsec) as u32);
                    Some(std::time::Duration::new(sec, nsec))
                } else {
                    None
                }
            }
        }
        #[derive(Copy, Clone)]
        pub struct MistInstant {
            t: Timespec
        }
        impl MistInstant {
            pub fn now() -> Self {
                Self {t: now(libc::CLOCK_BOOTTIME)}
            }
        }
        #[cfg(not(any(target_os = "dragonfly", target_os = "espidf")))]
        pub type clock_t = libc::c_int;
        #[cfg(any(target_os = "dragonfly", target_os = "espidf"))]
        pub type clock_t = libc::c_ulong;
        fn now(clock: clock_t) -> Timespec {
            let mut t = Timespec {t: libc::timespec {tv_sec: 0, tv_nsec: 0}};
            let r = unsafe {libc::clock_gettime(clock, &mut t.t)};
            if r == -1 {
                panic!("couldn't clock_gettime");
            }
            t
        }
        impl std::ops::Sub<MistInstant> for MistInstant {
            type Output = std::time::Duration;
            fn sub(self, other: MistInstant) -> Self::Output {
                self.t.sub_timespec(&other.t).expect("overflow when subtracting instants")
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

