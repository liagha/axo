use {
    crate::{
        data::Scale,
        internal::asm,
        format::Debug,
    }
};

pub use {
    core::time::Duration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerError {
    AlreadyRunning,
    NotRunning,
    InvalidDuration,
    Overflow,
    StorageFull,
}

pub type TimerResult<T> = Result<T, TimerError>;

pub trait TimeValue: Copy + Clone + PartialOrd + PartialEq + Ord + Debug {
    const ZERO: Self;
    const ONE: Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn saturating_add(self, other: Self) -> Self;
    fn saturating_sub(self, other: Self) -> Self;
    fn saturating_mul(self, scalar: u32) -> Self;
    fn saturating_div(self, scalar: u32) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn from_u32(value: u32) -> Self;
    fn as_u32(self) -> u32;
}

impl TimeValue for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn max_value() -> Self { u64::MAX }
    fn saturating_add(self, other: Self) -> Self { self.saturating_add(other) }
    fn saturating_sub(self, other: Self) -> Self { self.saturating_sub(other) }
    fn saturating_mul(self, scalar: u32) -> Self { self.saturating_mul(scalar as u64) }
    fn saturating_div(self, scalar: u32) -> Self { self / (scalar as u64) }
    fn checked_add(self, other: Self) -> Option<Self> { self.checked_add(other) }
    fn checked_sub(self, other: Self) -> Option<Self> { self.checked_sub(other) }
    fn from_u32(value: u32) -> Self { value as u64 }
    fn as_u32(self) -> u32 { self.min(u32::MAX as u64) as u32 }
}

impl TimeValue for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn max_value() -> Self { u32::MAX }
    fn saturating_add(self, other: Self) -> Self { self.saturating_add(other) }
    fn saturating_sub(self, other: Self) -> Self { self.saturating_sub(other) }
    fn saturating_mul(self, scalar: u32) -> Self { self.saturating_mul(scalar) }
    fn saturating_div(self, scalar: u32) -> Self { self / scalar }
    fn checked_add(self, other: Self) -> Option<Self> { self.checked_add(other) }
    fn checked_sub(self, other: Self) -> Option<Self> { self.checked_sub(other) }
    fn from_u32(value: u32) -> Self { value }
    fn as_u32(self) -> u32 { self }
}

impl TimeValue for u128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn max_value() -> Self { u128::MAX }
    fn saturating_add(self, other: Self) -> Self { self.saturating_add(other) }
    fn saturating_sub(self, other: Self) -> Self { self.saturating_sub(other) }
    fn saturating_mul(self, scalar: u32) -> Self { self.saturating_mul(scalar as u128) }
    fn saturating_div(self, scalar: u32) -> Self { self / (scalar as u128) }
    fn checked_add(self, other: Self) -> Option<Self> { self.checked_add(other) }
    fn checked_sub(self, other: Self) -> Option<Self> { self.checked_sub(other) }
    fn from_u32(value: u32) -> Self { value as u128 }
    fn as_u32(self) -> u32 { self.min(u32::MAX as u128) as u32 }
}

pub trait TimeSource<T: TimeValue> : Sized {
    fn now(&self) -> T;
    fn resolution(&self) -> T;
}

pub trait LapStorage<T: TimeValue> {
    fn push(&mut self, value: T) -> Result<(), TimerError>;
    fn clear(&mut self);
    fn get(&self, index: usize) -> Option<T>;
    fn len(&self) -> usize;
    fn is_full(&self) -> bool;
    fn as_slice(&self) -> &[T];
}

pub struct ArrayLapStorage<T: TimeValue, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: TimeValue, const N: usize> ArrayLapStorage<T, N> {
    pub const fn new() -> Self {
        Self {
            data: [T::ZERO; N],
            len: 0,
        }
    }
}

impl<T: TimeValue, const N: usize> LapStorage<T> for ArrayLapStorage<T, N> {
    fn push(&mut self, value: T) -> Result<(), TimerError> {
        if self.is_full() {
            for i in 1..N {
                self.data[i - 1] = self.data[i];
            }
            self.data[N - 1] = value;
        } else {
            self.data[self.len] = value;
            self.len += 1;
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.len = 0;
        self.data.fill(T::zero());
    }

    fn get(&self, index: usize) -> Option<T> {
        if index < self.len {
            Some(self.data[index])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_full(&self) -> bool {
        self.len >= N
    }

    fn as_slice(&self) -> &[T] {
        &self.data[0..self.len]
    }
}

pub struct RingLapStorage<T: TimeValue, const N: usize> {
    data: [T; N],
    head: usize,
    len: usize,
}

impl<T: TimeValue, const N: usize> RingLapStorage<T, N> {
    pub const fn new() -> Self {
        Self {
            data: [T::ZERO; N],
            head: 0,
            len: 0,
        }
    }
}

impl<T: TimeValue, const N: usize> LapStorage<T> for RingLapStorage<T, N> {
    fn push(&mut self, value: T) -> Result<(), TimerError> {
        self.data[self.head] = value;
        self.head = (self.head + 1) % N;
        if self.len < N {
            self.len += 1;
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
        self.data.fill(T::zero());
    }

    fn get(&self, index: usize) -> Option<T> {
        if index < self.len {
            let actual_index = if self.len == N {
                (self.head + index) % N
            } else {
                index
            };
            Some(self.data[actual_index])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_full(&self) -> bool {
        self.len >= N
    }

    fn as_slice(&self) -> &[T] {
        if self.len == N {
            &[]
        } else {
            &self.data[0..self.len]
        }
    }
}

pub trait CpuCycleCounter<T: TimeValue> {
    fn read_cycles() -> T;
    fn frequency() -> T;
}

#[cfg(target_arch = "x86_64")]
pub struct X86CycleCounter;

#[cfg(target_arch = "x86_64")]
impl CpuCycleCounter<u64> for X86CycleCounter {
    fn read_cycles() -> u64 {
        let low: u32;
        let high: u32;
        unsafe {
            asm!("rdtsc", out("eax") low, out("edx") high, options(nostack, nomem));
        }
        ((high as u64) << 32) | (low as u64)
    }

    fn frequency() -> u64 {
        3_000_000_000
    }
}

#[cfg(target_arch = "x86_64")]
pub struct CPUCycleSource;

#[cfg(target_arch = "x86_64")]
impl CPUCycleSource {
    pub const fn new() -> Self {
        CPUCycleSource
    }
}

#[cfg(target_arch = "x86_64")]
impl TimeSource<u64> for CPUCycleSource {
    fn now(&self) -> u64 {
        X86CycleCounter::read_cycles()
    }

    fn resolution(&self) -> u64 {
        X86CycleCounter::frequency()
    }
}

#[cfg(target_arch = "aarch64")]
pub struct ARMCycleCounter;

#[cfg(target_arch = "aarch64")]
impl CpuCycleCounter<u64> for ARMCycleCounter {
    fn read_cycles() -> u64 {
        let cnt: u64;
        unsafe {
            asm!("mrs {}, cntvct_el0", out(reg) cnt, options(nostack, nomem));
        }
        cnt
    }

    fn frequency() -> u64 {
        let freq: u64;
        unsafe {
            asm!("mrs {}, cntfrq_el0", out(reg) freq, options(nostack, nomem));
        }
        freq
    }
}

#[cfg(target_arch = "aarch64")]
pub struct ARMGenericTimerSource;

#[cfg(target_arch = "aarch64")]
impl ARMGenericTimerSource {
    pub const fn new() -> Self {
        ARMGenericTimerSource
    }
}

#[cfg(target_arch = "aarch64")]
impl TimeSource<u64> for ARMGenericTimerSource {
    fn now(&self) -> u64 {
        ARMCycleCounter::read_cycles()
    }

    fn resolution(&self) -> u64 {
        ARMCycleCounter::frequency()
    }
}

#[cfg(target_arch = "riscv64")]
pub struct RISCVCycleCounter;

#[cfg(target_arch = "riscv64")]
impl CpuCycleCounter<u64> for RISCVCycleCounter {
    fn read_cycles() -> u64 {
        let cycles: u64;
        unsafe {
            asm!("rdcycle {}", out(reg) cycles, options(nostack, nomem));
        }
        cycles
    }

    fn frequency() -> u64 {
        1_000_000
    }
}

#[cfg(target_arch = "riscv64")]
pub struct RISCVCycleSource;

#[cfg(target_arch = "riscv64")]
impl RISCVCycleSource {
    pub const fn new() -> Self {
        RISCVCycleSource
    }
}

#[cfg(target_arch = "riscv64")]
impl TimeSource<u64> for RISCVCycleSource {
    fn now(&self) -> u64 {
        RISCVCycleCounter::read_cycles()
    }

    fn resolution(&self) -> u64 {
        RISCVCycleCounter::frequency()
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
pub struct GenericCycleCounter;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
impl<T: TimeValue> CpuCycleCounter<T> for GenericCycleCounter {
    fn read_cycles() -> T {
        T::zero()
    }

    fn frequency() -> T {
        T::max_value()
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
impl<T: TimeValue> TimeSource<T> for DummyTimeSource {
    fn now(&self) -> T {
        T::zero()
    }

    fn resolution(&self) -> T {
        T::max_value()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
}

pub struct Timer<T, S, L>
where
    T: TimeValue,
    S: TimeSource<T>,
    L: LapStorage<T>,
{
    source: S,
    storage: L,
    state: TimerState,
    start_time: T,
    accumulated: T,
    target_duration: Option<T>,
}

impl<T, S, L> Timer<T, S, L>
where
    T: TimeValue,
    S: TimeSource<T>,
    L: LapStorage<T>,
{
    pub fn new(time_source: S, lap_storage: L) -> Self {
        Timer {
            source: time_source,
            storage: lap_storage,
            state: TimerState::Stopped,
            start_time: T::zero(),
            accumulated: T::zero(),
            target_duration: None,
        }
    }

    pub fn start(&mut self) -> TimerResult<()> {
        if self.state == TimerState::Running {
            return Err(TimerError::AlreadyRunning);
        }

        self.state = TimerState::Running;
        self.start_time = self.source.now();
        self.accumulated = T::zero();
        self.storage.clear();

        Ok(())
    }

    pub fn stop(&mut self) -> TimerResult<T> {
        if self.state == TimerState::Stopped {
            return Err(TimerError::NotRunning);
        }
        let elapsed = self.elapsed()?;
        self.state = TimerState::Stopped;
        self.accumulated = T::zero();
        Ok(elapsed)
    }

    pub fn pause(&mut self) -> TimerResult<T> {
        match self.state {
            TimerState::Running => {
                let elapsed = self.elapsed()?;
                self.accumulated = elapsed;
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
                self.start_time = self.source.now();
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Running => Err(TimerError::AlreadyRunning),
            TimerState::Stopped => Err(TimerError::NotRunning),
        }
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Stopped;
        self.start_time = T::zero();
        self.accumulated = T::zero();
        self.storage.clear();
    }

    pub fn elapsed(&self) -> TimerResult<T> {
        match self.state {
            TimerState::Stopped => {
                if self.accumulated > T::zero() {
                    Ok(self.accumulated)
                } else {
                    Err(TimerError::NotRunning)
                }
            }
            TimerState::Paused => Ok(self.accumulated),
            TimerState::Running => {
                let now = self.source.now();
                if now < self.start_time {
                    Err(TimerError::Overflow)
                } else {
                    now.checked_sub(self.start_time)
                        .and_then(|diff| self.accumulated.checked_add(diff))
                        .ok_or(TimerError::Overflow)
                }
            }
        }
    }

    pub fn set_duration(&mut self, duration: T) -> TimerResult<()> {
        if duration == T::zero() {
            return Err(TimerError::InvalidDuration);
        }
        self.target_duration = Some(duration);
        Ok(())
    }

    pub fn clear_duration(&mut self) {
        self.target_duration = None;
    }

    pub fn is_expired(&self) -> bool {
        self.target_duration
            .map(|duration| self.elapsed().map_or(false, |elapsed| elapsed >= duration))
            .unwrap_or(false)
    }

    pub fn lap(&mut self) -> TimerResult<T> {
        if self.state != TimerState::Running {
            return Err(TimerError::NotRunning);
        }
        let elapsed = self.elapsed()?;
        self.storage.push(elapsed)?;
        Ok(elapsed)
    }

    pub fn laps(&self) -> &[T] {
        self.storage.as_slice()
    }

    pub fn lap_count(&self) -> usize {
        self.storage.len()
    }

    pub fn get_lap(&self, index: usize) -> Option<T> {
        self.storage.get(index)
    }

    pub fn state(&self) -> TimerState {
        self.state
    }

    pub fn to_seconds(&self, time: T) -> T {
        let resolution = self.source.resolution();
        let resolution_u32 = resolution.as_u32().max(1);
        let scale_factor = (resolution_u32 / 1000).max(1);
        time.saturating_div(scale_factor)
    }

    pub fn to_milliseconds(&self, time: T) -> T {
        let resolution = self.source.resolution();
        let resolution_u32 = resolution.as_u32().max(1);

        if resolution_u32 >= 1_000_000 {
            let scale_factor = (resolution_u32 / 1_000_000).max(1);
            time.saturating_mul(1000).saturating_div(scale_factor)
        } else {
            time.saturating_mul(1000)
        }
    }

    pub fn to_microseconds(&self, time: T) -> T {
        let resolution = self.source.resolution();
        let resolution_u32 = resolution.as_u32().max(1);

        if resolution_u32 >= 1_000_000 {
            let scale_factor = resolution_u32 / 1_000_000;
            time.saturating_mul(1_000_000).saturating_div(scale_factor.max(1))
        } else {
            time.saturating_mul(1_000_000)
        }
    }

    pub fn to_nanoseconds(&self, time: T) -> T {
        let resolution = self.source.resolution();
        let resolution_u32 = resolution.as_u32().max(1);

        if resolution_u32 >= 1_000_000_000 {
            let scale_factor = resolution_u32 / 1_000_000_000;
            time.saturating_mul(1_000_000_000).saturating_div(scale_factor.max(1))
        } else {
            time.saturating_mul(1_000_000_000)
        }
    }

    pub fn remaining(&self) -> TimerResult<Option<T>> {
        self.target_duration
            .map(|duration| {
                let elapsed = self.elapsed()?;
                Ok(if elapsed >= duration {
                    T::zero()
                } else {
                    duration.saturating_sub(elapsed)
                })
            })
            .transpose()
    }
}

pub type DefaultTimer = Timer<u64, CPUCycleSource, ArrayLapStorage<u64, 32>>;

#[cfg(target_arch = "x86_64")]
pub type PlatformTimer<T, L> = Timer<T, CPUCycleSource, L>;

#[cfg(target_arch = "aarch64")]
pub type PlatformTimer<T, L> = Timer<T, ARMGenericTimerSource, L>;

#[cfg(target_arch = "riscv64")]
pub type PlatformTimer<T, L> = Timer<T, RISCVCycleSource, L>;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
pub type PlatformTimer<T, L> = Timer<T, DummyTimeSource, L>;

impl DefaultTimer {
    pub fn new_default() -> Self {
        Timer::new(CPUCycleSource::new(), ArrayLapStorage::new())
    }
}

pub trait TimerCallback<T: TimeValue> {
    fn on_tick(&mut self, elapsed: T, remaining: Option<T>);
    fn on_complete(&mut self);
}

pub struct CallbackTimer<T, S, L, C>
where
    T: TimeValue,
    S: TimeSource<T>,
    L: LapStorage<T>,
    C: TimerCallback<T>,
{
    timer: Timer<T, S, L>,
    callback: C,
    tick_interval: T,
    last_tick: T,
}

impl<T, S, L, C> CallbackTimer<T, S, L, C>
where
    T: TimeValue,
    S: TimeSource<T>,
    L: LapStorage<T>,
    C: TimerCallback<T>,
{
    pub fn new(time_source: S, lap_storage: L, callback: C, tick_interval: T) -> Self {
        CallbackTimer {
            timer: Timer::new(time_source, lap_storage),
            callback,
            tick_interval,
            last_tick: T::zero(),
        }
    }

    pub fn update(&mut self) -> TimerResult<()> {
        if self.timer.state() != TimerState::Running {
            return Ok(());
        }
        let elapsed = self.timer.elapsed()?;
        let remaining = self.timer.remaining()?;
        if elapsed.saturating_sub(self.last_tick) >= self.tick_interval {
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
            self.last_tick = T::zero();
        }
        result
    }

    pub fn stop(&mut self) -> TimerResult<T> {
        self.timer.stop()
    }

    pub fn set_duration(&mut self, duration: T) -> TimerResult<()> {
        self.timer.set_duration(duration)
    }
}

pub struct CountdownTimer<T, S, L>
where
    T: TimeValue,
    S: TimeSource<T>,
    L: LapStorage<T>,
{
    timer: Timer<T, S, L>,
}

impl<T, S, L> CountdownTimer<T, S, L>
where
    T: TimeValue,
    S: TimeSource<T>,
    L: LapStorage<T>,
{
    pub fn new(time_source: S, lap_storage: L, duration: T) -> TimerResult<Self> {
        let mut timer = Timer::new(time_source, lap_storage);
        timer.set_duration(duration)?;
        Ok(CountdownTimer { timer })
    }

    pub fn start(&mut self) -> TimerResult<()> {
        self.timer.start()
    }

    pub fn stop(&mut self) -> TimerResult<T> {
        self.timer.stop()
    }

    pub fn remaining(&self) -> TimerResult<T> {
        self.timer
            .remaining()?
            .ok_or(TimerError::InvalidDuration)
    }

    pub fn is_expired(&self) -> bool {
        self.timer.is_expired()
    }
}

impl<S, L> CountdownTimer<u64, S, L>
where
    S: TimeSource<u64>,
    L: LapStorage<u64>,
{
    pub fn format_remaining(&self) -> TimerResult<(u64, u64)> {
        let remaining_ms = self.timer.to_milliseconds(self.remaining()?);
        let seconds = (remaining_ms / 1000) % 60;
        let minutes = (remaining_ms / 1000) / 60;
        Ok((minutes, seconds))
    }
}