//! Process resource probe: peak memory and CPU time, by sampling.
//!
//! Memory is the resident set (working set on Windows) sampled every ~25 ms, so very short
//! spikes can be missed: reported peaks are lower bounds, not exact maxima. CPU time is the
//! process' accumulated CPU time (user + kernel, all threads), so CPU time divided by wall
//! time gives the average number of cores kept busy.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

const SAMPLE_EVERY: Duration = Duration::from_millis(25);

pub struct ProcessProbe {
    sys: System,
    pid: Pid,
    peak_bytes: u64,
    first_cpu_ms: Option<u64>,
    last_cpu_ms: u64,
    samples: u32,
}

impl ProcessProbe {
    pub fn new(pid: u32) -> Self {
        Self {
            sys: System::new(),
            pid: Pid::from_u32(pid),
            peak_bytes: 0,
            first_cpu_ms: None,
            last_cpu_ms: 0,
            samples: 0,
        }
    }

    pub fn for_current_process() -> Self {
        Self::new(std::process::id())
    }

    pub fn sample(&mut self) {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.pid]),
            true,
            ProcessRefreshKind::everything(),
        );
        if let Some(p) = self.sys.process(self.pid) {
            self.peak_bytes = self.peak_bytes.max(p.memory());
            let cpu = p.accumulated_cpu_time();
            self.first_cpu_ms.get_or_insert(cpu);
            self.last_cpu_ms = cpu;
            self.samples += 1;
        }
    }

    /// Peak resident memory in MB (None if the process was never observed).
    pub fn peak_mb(&self) -> Option<f64> {
        (self.samples > 0).then(|| self.peak_bytes as f64 / (1024.0 * 1024.0))
    }

    /// CPU time consumed between the first and the last observation, in ms.
    /// For a child process started just before the first sample this is its whole CPU time
    /// (minus what ran before the first sample, at most a few tens of ms).
    pub fn cpu_ms(&self) -> Option<u64> {
        (self.samples > 0).then(|| self.last_cpu_ms.saturating_sub(self.first_cpu_ms.unwrap_or(0)))
    }
}

/// Samples the CURRENT process on a background thread until `finish` is called.
pub struct SelfSampler {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<ProcessProbe>>,
}

/// Dropping a sampler without calling `finish` (for example on an early error) still stops
/// its thread.
impl Drop for SelfSampler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl SelfSampler {
    pub fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            let mut probe = ProcessProbe::for_current_process();
            probe.sample();
            while !flag.load(Ordering::Relaxed) {
                thread::sleep(SAMPLE_EVERY);
                probe.sample();
            }
            probe.sample();
            probe
        });
        Self { stop, handle: Some(handle) }
    }

    /// Stops sampling and returns (peak MB, CPU ms) over the window.
    pub fn finish(mut self) -> (Option<f64>, Option<u64>) {
        self.stop.store(true, Ordering::Relaxed);
        match self.handle.take().map(JoinHandle::join) {
            Some(Ok(p)) => (p.peak_mb(), p.cpu_ms()),
            _ => (None, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sees_memory_growth_and_cpu_time_of_the_current_process() {
        let sampler = SelfSampler::start();
        // Allocate and touch ~64 MB, and burn some CPU.
        let mut big = vec![0u8; 64 * 1024 * 1024];
        for (i, b) in big.iter_mut().enumerate().step_by(4096) {
            *b = (i % 251) as u8;
        }
        let mut x = 0u64;
        let t = std::time::Instant::now();
        while t.elapsed() < Duration::from_millis(300) {
            x = x.wrapping_add(std::hint::black_box(1));
        }
        std::hint::black_box((&big, x));
        let (mem, cpu) = sampler.finish();
        let mem = mem.expect("memory observed");
        assert!(mem > 60.0, "peak {mem} MB should include the 64 MB buffer");
        assert!(cpu.expect("cpu observed") >= 100, "cpu ms should reflect the busy loop");
    }

    #[test]
    fn a_dropped_sampler_stops_its_thread() {
        let stop;
        {
            let s = SelfSampler::start();
            stop = Arc::clone(&s.stop);
            assert!(!stop.load(Ordering::Relaxed));
        }
        assert!(stop.load(Ordering::Relaxed));
    }

    #[test]
    fn a_missing_process_reports_nothing() {
        let mut p = ProcessProbe::new(u32::MAX - 1);
        p.sample();
        assert_eq!(p.peak_mb(), None);
        assert_eq!(p.cpu_ms(), None);
    }
}
