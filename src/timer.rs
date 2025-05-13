#![allow(dead_code)]

use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    AlreadyRunning,
    NotRunning,
    InvalidDuration,
    Overflow,
}

pub type TimerResult<T> = Result<T, TimerError>;

pub trait TimeSource {
    fn now(&self) -> u64;
    fn resolution(&self) -> u64; 
}

#[cfg(target_arch = "x86_64")]
pub struct CPUCycleSource;

#[cfg(target_arch = "x86_64")]
impl CPUCycleSource {
    pub const fn new() -> Self {
        CPUCycleSource
    }

    #[inline(always)]
    fn read_cycle_counter() -> u64 {
        let low: u32;
        let high: u32;
        unsafe {
            asm!("rdtsc", out("eax") low, out("edx") high, options(nostack, nomem));
        }
        ((high as u64) << 32) | (low as u64)
    }
}

#[cfg(target_arch = "x86_64")]
impl TimeSource for CPUCycleSource {
    fn now(&self) -> u64 {
        Self::read_cycle_counter()
    }

    fn resolution(&self) -> u64 {
        1_000_000_000
    }
}

#[cfg(target_arch = "aarch64")]
pub struct ARMGenericTimerSource;

#[cfg(target_arch = "aarch64")]
impl ARMGenericTimerSource {
    pub const fn new() -> Self {
        ARMGenericTimerSource
    }

    #[inline(always)]
    fn read_counter() -> u64 {
        let cnt: u64;
        unsafe {
            asm!("mrs {}, cntvct_el0", out(reg) cnt, options(nostack, nomem));
        }
        cnt
    }
}

#[cfg(target_arch = "aarch64")]
impl TimeSource for ARMGenericTimerSource {
    fn now(&self) -> u64 {
        Self::read_counter()
    }

    fn resolution(&self) -> u64 {
        let freq: u64;
        unsafe {
            asm!("mrs {}, cntfrq_el0", out(reg) freq, options(nostack, nomem));
        }
        freq
    }
}

#[cfg(target_arch = "riscv64")]
pub struct RISCVCycleSource;

#[cfg(target_arch = "riscv64")]
impl RISCVCycleSource {
    pub const fn new() -> Self {
        RISCVCycleSource
    }

    #[inline(always)]
    fn read_cycle_counter() -> u64 {
        let cycles: u64;
        unsafe {
            asm!("rdcycle {}", out(reg) cycles, options(nostack, nomem));
        }
        cycles
    }
}

#[cfg(target_arch = "riscv64")]
impl TimeSource for RISCVCycleSource {
    fn now(&self) -> u64 {
        Self::read_cycle_counter()
    }

    fn resolution(&self) -> u64 {
        1_000_000
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
pub struct DummyTimeSource;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
impl DummyTimeSource {
    pub const fn new() -> Self {
        DummyTimeSource
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
impl TimeSource for DummyTimeSource {
    fn now(&self) -> u64 {
        0
    }

    fn resolution(&self) -> u64 {
        1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
}

pub struct Timer<T: TimeSource> {
    time_source: T,
    state: TimerState,
    start_time: u64,
    elapsed_before_pause: u64,
    duration: Option<u64>,
    laps: [u64; 32], 
    lap_count: usize,
}

impl<T: TimeSource> Timer<T> {
    pub fn new(time_source: T) -> Self {
        let mut timer = Timer {
            time_source,
            state: TimerState::Stopped,
            start_time: 0,
            elapsed_before_pause: 0,
            duration: None,
            laps: [0; 32],
            lap_count: 0,
        };

        let _ = timer.start();

        timer
    }

    pub fn start(&mut self) -> TimerResult<()> {
        if self.state == TimerState::Running {
            return Err(TimerError::AlreadyRunning);
        }
        self.state = TimerState::Running;
        self.start_time = self.time_source.now();
        self.elapsed_before_pause = 0;
        self.lap_count = 0;
        Ok(())
    }

    pub fn stop(&mut self) -> TimerResult<u64> {
        if self.state == TimerState::Stopped {
            return Err(TimerError::NotRunning);
        }
        let elapsed = self.elapsed()?;
        self.state = TimerState::Stopped;
        self.elapsed_before_pause = 0;
        Ok(elapsed)
    }

    pub fn pause(&mut self) -> TimerResult<u64> {
        match self.state {
            TimerState::Running => {
                let elapsed = self.elapsed()?;
                self.elapsed_before_pause = elapsed;
                self.state = TimerState::Paused;
                Ok(elapsed)
            }
            TimerState::Paused => Err(TimerError::AlreadyRunning),
            TimerState::Stopped => Err(TimerError::NotRunning),
        }
    }

    pub fn resume(&mut self) -> TimerResult<()> {
        match self.state {
            TimerState::Paused => {
                self.start_time = self.time_source.now();
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Running => Err(TimerError::AlreadyRunning),
            TimerState::Stopped => Err(TimerError::NotRunning),
        }
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Stopped;
        self.start_time = 0;
        self.elapsed_before_pause = 0;
        self.lap_count = 0;
    }

    pub fn elapsed(&self) -> TimerResult<u64> {
        match self.state {
            TimerState::Stopped => {
                if self.elapsed_before_pause > 0 {
                    Ok(self.elapsed_before_pause)
                } else {
                    Err(TimerError::NotRunning)
                }
            }
            TimerState::Paused => Ok(self.elapsed_before_pause),
            TimerState::Running => {
                let now = self.time_source.now();
                if now < self.start_time {
                    Err(TimerError::Overflow)
                } else {
                    Ok(self.elapsed_before_pause + (now - self.start_time))
                }
            }
        }
    }

    pub fn set_duration(&mut self, duration: u64) -> TimerResult<()> {
        if duration == 0 {
            return Err(TimerError::InvalidDuration);
        }
        self.duration = Some(duration);
        Ok(())
    }

    pub fn clear_duration(&mut self) {
        self.duration = None;
    }

    pub fn is_expired(&self) -> bool {
        self.duration
            .map(|duration| self.elapsed().map_or(false, |elapsed| elapsed >= duration))
            .unwrap_or(false)
    }

    pub fn lap(&mut self) -> TimerResult<u64> {
        if self.state != TimerState::Running {
            return Err(TimerError::NotRunning);
        }
        if self.lap_count >= self.laps.len() {
            for i in 1..self.laps.len() {
                self.laps[i - 1] = self.laps[i];
            }
            self.lap_count = self.laps.len() - 1;
        }
        let elapsed = self.elapsed()?;
        self.laps[self.lap_count] = elapsed;
        self.lap_count += 1;
        Ok(elapsed)
    }

    pub fn laps(&self) -> &[u64] {
        &self.laps[0..self.lap_count]
    }

    pub fn state(&self) -> TimerState {
        self.state
    }

    pub fn to_seconds(&self, time: u64) -> u64 {
        time / self.time_source.resolution()
    }

    pub fn to_milliseconds(&self, time: u64) -> u64 {
        time * 1_000 / self.time_source.resolution()
    }

    pub fn to_microseconds(&self, time: u64) -> u64 {
        time * 1_000_000 / self.time_source.resolution()
    }

    pub fn to_nanoseconds(&self, time: u64) -> u64 {
        time * 1_000_000_000 / self.time_source.resolution()
    }

    pub fn remaining(&self) -> TimerResult<Option<u64>> {
        self.duration
            .map(|duration| {
                let elapsed = self.elapsed()?;
                Ok(if elapsed >= duration {
                    0
                } else {
                    duration - elapsed
                })
            })
            .transpose()
    }
}

pub trait TimerCallback {
    fn on_tick(&mut self, elapsed: u64, remaining: Option<u64>);
    fn on_complete(&mut self);
}

pub struct CallbackTimer<T: TimeSource, C: TimerCallback> {
    timer: Timer<T>,
    callback: C,
    tick_interval: u64,
    last_tick: u64,
}

impl<T: TimeSource, C: TimerCallback> CallbackTimer<T, C> {
    pub fn new(time_source: T, callback: C, tick_interval: u64) -> Self {
        CallbackTimer {
            timer: Timer::new(time_source),
            callback,
            tick_interval,
            last_tick: 0,
        }
    }

    pub fn update(&mut self) -> TimerResult<()> {
        if self.timer.state() != TimerState::Running {
            return Ok(());
        }
        let elapsed = self.timer.elapsed()?;
        let remaining = self.timer.remaining()?;
        if elapsed - self.last_tick >= self.tick_interval {
            self.last_tick = elapsed;
            self.callback.on_tick(elapsed, remaining);
        }
        if self.timer.is_expired() {
            self.callback.on_complete();
            self.timer.stop()?;
        }
        Ok(())
    }

    pub fn start(&mut self) -> TimerResult<()> {
        let result = self.timer.start();
        if result.is_ok() {
            self.last_tick = 0;
        }
        result
    }

    pub fn stop(&mut self) -> TimerResult<u64> {
        self.timer.stop()
    }

    pub fn set_duration(&mut self, duration: u64) -> TimerResult<()> {
        self.timer.set_duration(duration)
    }
}

pub struct CountdownTimer<T: TimeSource> {
    timer: Timer<T>,
}

impl<T: TimeSource> CountdownTimer<T> {
    pub fn new(time_source: T, duration: u64) -> TimerResult<Self> {
        let mut timer = Timer::new(time_source);
        timer.set_duration(duration)?;
        Ok(CountdownTimer { timer })
    }

    pub fn start(&mut self) -> TimerResult<()> {
        self.timer.start()
    }

    pub fn stop(&mut self) -> TimerResult<u64> {
        self.timer.stop()
    }

    pub fn remaining(&self) -> TimerResult<u64> {
        self.timer
            .remaining()?
            .ok_or(TimerError::InvalidDuration)
    }

    pub fn is_expired(&self) -> bool {
        self.timer.is_expired()
    }

    pub fn format_remaining(&self) -> TimerResult<(u64, u64)> {
        let remaining_ms = self.timer.to_milliseconds(self.remaining()?);
        let seconds = (remaining_ms / 1000) % 60;
        let minutes = (remaining_ms / 1000) / 60;
        Ok((minutes, seconds))
    }
}