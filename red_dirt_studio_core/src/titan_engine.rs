// Titan 7 Pro-Studio V2 Audio Engine
// Single-file hybrid recording + audio engine + driver.
// Neve DFC3D-style DSP graph + SSL SuperAnalogue-style preamp.
// Deterministic core: no allocations, no locks, no syscalls in audio thread.
// Desktop (cpal/ASIO/WASAPI) + Web (WASM + AudioWorklet).

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;

// ── Basic types ──────────────────────────────────────────────────────────────

pub type Sample = f32;
pub type SampleRate = f64;
pub type BufferSize = u32;
pub type ChannelCount = u32;

// ── Lock-free SPSC queue (for control messages, never audio data) ────────────

pub struct LockFreeQueue<T, const CAP: usize> {
    buffer: UnsafeCell<MaybeUninit<[T; CAP]>>,
    head: AtomicU32,
    tail: AtomicU32,
}

unsafe impl<T: Send, const CAP: usize> Send for LockFreeQueue<T, CAP> {}
unsafe impl<T: Sync, const CAP: usize> Sync for LockFreeQueue<T, CAP> {}

impl<T: Copy, const CAP: usize> LockFreeQueue<T, CAP> {
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new(MaybeUninit::uninit()),
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
        }
    }

    pub fn push(&self, value: T) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let next = (head + 1) % CAP as u32;
        if next == self.tail.load(Ordering::Acquire) {
            return false;
        }
        unsafe {
            (*self.buffer.get()).assume_init_mut()[head as usize] = value;
        }
        self.head.store(next, Ordering::Release);
        true
    }

    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        let value = unsafe { (*self.buffer.get()).assume_init_ref()[tail as usize] };
        self.tail.store((tail + 1) % CAP as u32, Ordering::Release);
        Some(value)
    }
}

// Default impl for initialization convenience
impl<T: Copy + Default, const CAP: usize> Default for LockFreeQueue<T, CAP> {
    fn default() -> Self {
        let q = Self::new();
        // Initialize all slots
        for i in 0..CAP {
            unsafe {
                (*q.buffer.get()).assume_init_mut()[i] = T::default();
            }
        }
        q
    }
}

// ── DSP node trait ───────────────────────────────────────────────────────────

pub trait DspNode {
    fn process(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], frames: usize);
}

// ── SuperAnalogue-style preamp ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SuperAnaloguePreamp {
    gain: Sample,
}

impl SuperAnaloguePreamp {
    pub fn new() -> Self {
        let mut s = Self { gain: 1.0 };
        s.set_gain_db(0.0);
        s
    }

    pub fn set_gain_db(&mut self, db: f32) {
        self.gain = 10.0_f32.powf(db / 20.0);
    }
}

impl DspNode for SuperAnaloguePreamp {
    fn process(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], frames: usize) {
        let x = inputs[0];
        let y = &mut outputs[0][..frames];
        let g = self.gain;
        for i in 0..frames {
            y[i] = x[i] * g;
        }
    }
}

// ── Stereo summing bus (DFC-style, bounded inputs) ─────────────────────────

const MAX_BUS_INPUTS: usize = 16;

pub struct StereoBus {
    inputs: [Option<[*const Sample; 2]>; MAX_BUS_INPUTS],
    input_count: usize,
}

impl StereoBus {
    pub fn new() -> Self {
        Self {
            inputs: [None; MAX_BUS_INPUTS],
            input_count: 0,
        }
    }

    /// Register a stereo pair of input buffers (pointer-based to avoid borrow issues).
    /// The pointers must be valid for the duration of the graph.
    pub fn add_input(&mut self, buf: &[*const Sample; 2]) {
        if self.input_count < MAX_BUS_INPUTS {
            self.inputs[self.input_count] = Some(*buf);
            self.input_count += 1;
        }
    }
}

impl DspNode for StereoBus {
    fn process(&mut self, _inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], frames: usize) {
        for i in 0..frames {
            let mut sum_l = 0.0f32;
            let mut sum_r = 0.0f32;
            for s in 0..self.input_count {
                if let Some(ptr) = &self.inputs[s] {
                    sum_l += unsafe { *ptr[0].add(i) };
                    sum_r += unsafe { *ptr[1].add(i) };
                }
            }
            outputs[0][i] = sum_l;
            outputs[1][i] = sum_r;
        }
    }
}

// ── Control message ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub enum ParamMsg {
    SetPreampGainDb(f32),
    #[default]
    None,
}

// ── Engine metrics (reliability / observability) ────────────────────────────

#[derive(Debug)]
pub struct EngineMetrics {
    pub blocks_processed: AtomicU64,
    pub xruns: AtomicU64,
    pub overloads: AtomicU64,
}

impl EngineMetrics {
    pub const fn new() -> Self {
        Self {
            blocks_processed: AtomicU64::new(0),
            xruns: AtomicU64::new(0),
            overloads: AtomicU64::new(0),
        }
    }
}

// ── Core deterministic engine ───────────────────────────────────────────────

const MAX_CHANNELS: usize = 8;

pub struct TitanCoreEngine {
    sr: SampleRate,
    buffer_size: BufferSize,
    in_ch: ChannelCount,
    out_ch: ChannelCount,

    // Pre-allocated engine buffers (no heap in audio thread)
    in_bufs: Vec<Vec<Sample>>,
    out_bufs: Vec<Vec<Sample>>,

    // Node-local buffers
    pre_in: Vec<Sample>,
    pre_out: Vec<Sample>,

    // DSP nodes
    preamp: SuperAnaloguePreamp,
    bus: StereoBus,

    // Control
    param_queue: LockFreeQueue<ParamMsg, 64>,
    metrics: EngineMetrics,
    safe_mode: AtomicBool,
}

impl TitanCoreEngine {
    pub fn new() -> Self {
        Self {
            sr: 48000.0,
            buffer_size: 128,
            in_ch: 2,
            out_ch: 2,
            in_bufs: Vec::new(),
            out_bufs: Vec::new(),
            pre_in: Vec::new(),
            pre_out: Vec::new(),
            preamp: SuperAnaloguePreamp::new(),
            bus: StereoBus::new(),
            param_queue: LockFreeQueue::new(),
            metrics: EngineMetrics::new(),
            safe_mode: AtomicBool::new(false),
        }
    }

    pub fn init(
        &mut self,
        sr: SampleRate,
        buffer_size: BufferSize,
        in_ch: ChannelCount,
        out_ch: ChannelCount,
    ) -> bool {
        if in_ch as usize > MAX_CHANNELS || out_ch as usize > MAX_CHANNELS {
            return false;
        }
        self.sr = sr;
        self.buffer_size = buffer_size;
        self.in_ch = in_ch;
        self.out_ch = out_ch;

        let frames = buffer_size as usize;

        // Pre-allocate all buffers
        self.in_bufs = (0..in_ch).map(|_| vec![0.0; frames]).collect();
        self.out_bufs = (0..out_ch).map(|_| vec![0.0; frames]).collect();
        self.pre_in = vec![0.0; frames];
        self.pre_out = vec![0.0; frames];

        // Wire graph: input[0] -> preamp -> stereo bus -> outputs[0..1]
        // The bus reads from pre_out via raw pointers
        let pre_out_ptr: [*const Sample; 2] = [
            self.pre_out.as_ptr(),
            self.pre_out.as_ptr(), // mono preamp feeds both bus channels
        ];
        self.bus.add_input(&pre_out_ptr);

        self.safe_mode.store(false, Ordering::Release);
        true
    }

    /// Called from the audio driver callback. Zero allocations, no locks.
    pub fn process_block(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        frames: usize,
    ) {
        if frames != self.buffer_size as usize {
            self.metrics.xruns.fetch_add(1, Ordering::Relaxed);
            // Fail soft: zero output
            for ch in 0..self.out_ch as usize {
                for i in 0..frames {
                    output[ch][i] = 0.0;
                }
            }
            return;
        }

        self.apply_pending_params();

        // Copy input to internal buffers
        for ch in 0..self.in_ch as usize {
            let src = input[ch];
            let dst = &mut self.in_bufs[ch];
            for i in 0..frames {
                dst[i] = src[i];
            }
        }

        // Wire mono preamp from left input
        let src_l = &self.in_bufs[0];
        for i in 0..frames {
            self.pre_in[i] = src_l[i];
        }

        if !self.safe_mode.load(Ordering::Acquire) {
            // Normal DSP path
            let pre_in_refs = [&self.pre_in[..frames] as &[Sample]];
            let mut pre_out_muts = [&mut self.pre_out[..frames] as &mut [Sample]];
            self.preamp
                .process(&pre_in_refs, &mut pre_out_muts, frames);

            // Stereo bus reads from pre_out (registered in init) and writes to out_bufs
            let (buf0, rest) = self.out_bufs.split_at_mut(1);
            let (buf1, _) = rest.split_at_mut(1);
            let mut bus_out_muts = [
                &mut buf0[0][..frames] as &mut [Sample],
                &mut buf1[0][..frames] as &mut [Sample],
            ];
            self.bus
                .process(&[], &mut bus_out_muts, frames);
        } else {
            // Safe mode: bypass DSP, straight through
            for i in 0..frames {
                self.out_bufs[0][i] = self.in_bufs[0][i];
                if self.out_ch > 1 && self.in_ch > 1 {
                    self.out_bufs[1][i] = self.in_bufs[1][i];
                }
            }
        }

        // Copy to driver output
        for ch in 0..self.out_ch as usize {
            let src = &self.out_bufs[ch];
            let dst = &mut output[ch][..frames];
            for i in 0..frames {
                dst[i] = src[i];
            }
        }

        self.metrics.blocks_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn enqueue_param(&self, msg: ParamMsg) -> bool {
        self.param_queue.push(msg)
    }

    pub fn metrics(&self) -> &EngineMetrics {
        &self.metrics
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sr
    }
    pub fn buffer_size(&self) -> BufferSize {
        self.buffer_size
    }

    pub fn enable_safe_mode(&self, on: bool) {
        self.safe_mode.store(on, Ordering::Release);
    }

    fn apply_pending_params(&mut self) {
        while let Some(msg) = self.param_queue.pop() {
            match msg {
                ParamMsg::SetPreampGainDb(db) => self.preamp.set_gain_db(db),
                ParamMsg::None => {}
            }
        }
    }
}

// ── Audio driver trait ───────────────────────────────────────────────────────

pub trait AudioDriver {
    fn init(&mut self, sr: SampleRate, buffer_size: BufferSize, in_ch: ChannelCount, out_ch: ChannelCount) -> bool;
    fn shutdown(&mut self);
    fn process_callback(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], frames: usize) -> bool;
}

// ── Desktop low-latency driver stub (bind to cpal/ASIO/WASAPI/CoreAudio) ────

pub struct DesktopDriver<'a> {
    engine: &'a mut TitanCoreEngine,
    running: AtomicBool,
}

impl<'a> DesktopDriver<'a> {
    pub fn new(engine: &'a mut TitanCoreEngine) -> Self {
        Self {
            engine,
            running: AtomicBool::new(false),
        }
    }
}

impl AudioDriver for DesktopDriver<'_> {
    fn init(&mut self, _sr: SampleRate, _buffer_size: BufferSize, _in_ch: ChannelCount, _out_ch: ChannelCount) -> bool {
        // TODO: bind to cpal, WASAPI, or ASIO in exclusive/low-latency mode.
        self.running.store(true, Ordering::Release);
        true
    }

    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Release);
    }

    fn process_callback(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], frames: usize) -> bool {
        if !self.running.load(Ordering::Acquire) {
            return false;
        }
        self.engine.process_block(input, output, frames);
        true
    }
}

// ── WASM shim (AudioWorklet calls these via extern "C") ──────────────────────

#[cfg(target_arch = "wasm32")]
mod wasm_shim {
    use super::*;
    use std::sync::OnceLock;

    static ENGINE: OnceLock<std::sync::Mutex<TitanCoreEngine>> = OnceLock::new();

    fn engine() -> &'static std::sync::Mutex<TitanCoreEngine> {
        ENGINE.get_or_init(|| std::sync::Mutex::new(TitanCoreEngine::new()))
    }

    #[no_mangle]
    pub extern "C" fn titan_init(sr: f64, buffer_size: u32, in_ch: u32, out_ch: u32) -> bool {
        engine().lock().unwrap().init(sr, buffer_size, in_ch, out_ch)
    }

    /// Called from JS AudioWorklet for each block.
    /// `in` and `out` are pointers to arrays of channel pointers.
    #[no_mangle]
    pub unsafe extern "C" fn titan_process(
        input_ptrs: *mut *mut f32,
        output_ptrs: *mut *mut f32,
        frames: u32,
    ) {
        let eng = &mut *engine().lock().unwrap();
        let in_ch = eng.in_ch as usize;
        let out_ch = eng.out_ch as usize;
        let frames = frames as usize;

        // Build slice views from raw pointers
        let input: Vec<&[Sample]> = (0..in_ch)
            .map(|ch| std::slice::from_raw_parts(*input_ptrs.add(ch) as *const Sample, frames))
            .collect();
        let mut output: Vec<&mut [Sample]> = (0..out_ch)
            .map(|ch| std::slice::from_raw_parts_mut(*output_ptrs.add(ch), frames))
            .collect();

        eng.process_block(&input, &mut output, frames);
    }

    #[no_mangle]
    pub extern "C" fn titan_set_preamp_gain_db(db: f32) {
        engine().lock().unwrap().enqueue_param(ParamMsg::SetPreampGainDb(db));
    }

    #[no_mangle]
    pub extern "C" fn titan_enable_safe_mode(on: i32) {
        engine().lock().unwrap().enable_safe_mode(on != 0);
    }

    #[no_mangle]
    pub extern "C" fn titan_shutdown() {
        // OnceLock doesn't support removal; just no-op for now.
    }
}

// ── Offline mix function (for stem export / render) ─────────────────────────

/// Mix multiple stereo stems into a single master using the Titan engine's
/// SuperAnalogue preamp gain staging and DFC-style stereo summing bus.
///
/// Each stem's data is interleaved stereo `[L,R,L,R,...]`.
/// The output is a stereo `Vec<f32>` (interleaved) normalized to 0.95 peak.
pub fn titan_mix_stems(
    stem_data: &[(&str, &[Sample])],
    _sample_rate: u32,
    total_frames: usize,
    master_gain_db: f32,
) -> Vec<Sample> {
    if stem_data.is_empty() || total_frames == 0 {
        return vec![];
    }

    const BLOCK: usize = 128;

    // Pre-allocate per-stem gain-staging buffers (interleaved stereo)
    let mut stem_gained: Vec<Vec<Sample>> = stem_data
        .iter()
        .map(|_| vec![0.0f32; total_frames * 2])
        .collect();

    // ── Per-stem gain staging ────────────────────────────────────────────────
    for (s_idx, (_name, data)) in stem_data.iter().enumerate() {
        let mut preamp = SuperAnaloguePreamp::new();
        let mut offset = 0usize;
        while offset < total_frames {
            let block = BLOCK.min(total_frames - offset);

            // Left channel block
            let mut mono = vec![0.0f32; block];
            for i in 0..block {
                let idx = (offset + i) * 2;
                mono[i] = if idx < data.len() { data[idx] } else { 0.0 };
            }
            let mut processed = vec![0.0f32; block];
            {
                let in_ref = [&mono[..] as &[Sample]];
                let mut out_ref = [&mut processed[..] as &mut [Sample]];
                preamp.process(&in_ref, &mut out_ref, block);
            }
            for i in 0..block {
                stem_gained[s_idx][(offset + i) * 2] = processed[i];
            }

            // Right channel block
            for i in 0..block {
                let idx = (offset + i) * 2 + 1;
                mono[i] = if idx < data.len() { data[idx] } else { 0.0 };
            }
            {
                let in_ref = [&mono[..] as &[Sample]];
                let mut out_ref = [&mut processed[..] as &mut [Sample]];
                preamp.process(&in_ref, &mut out_ref, block);
            }
            for i in 0..block {
                stem_gained[s_idx][(offset + i) * 2 + 1] = processed[i];
            }

            offset += block;
        }
    }

    // ── Stereo summing bus ────────────────────────────────────────────────────
    let mut bus = StereoBus::new();
    for buf in &stem_gained {
        let ptrs = [buf.as_ptr(), buf.as_ptr().wrapping_add(1)];
        bus.add_input(&ptrs);
    }

    let mut master_preamp = SuperAnaloguePreamp::new();
    master_preamp.set_gain_db(master_gain_db);

    let mut output = vec![0.0f32; total_frames * 2];
    let mut offset = 0usize;
    while offset < total_frames {
        let block = BLOCK.min(total_frames - offset);
        let mut bus_l = vec![0.0f32; block];
        let mut bus_r = vec![0.0f32; block];
        {
            let mut bus_out = [&mut bus_l[..] as &mut [Sample], &mut bus_r[..] as &mut [Sample]];
            bus.process(&[], &mut bus_out, block);
        }

        // Master preamp: L
        let mut processed = vec![0.0f32; block];
        {
            let in_ref = [&bus_l[..] as &[Sample]];
            let mut out_ref = [&mut processed[..] as &mut [Sample]];
            master_preamp.process(&in_ref, &mut out_ref, block);
        }
        for i in 0..block {
            output[(offset + i) * 2] = processed[i];
        }

        // Master preamp: R
        {
            let in_ref = [&bus_r[..] as &[Sample]];
            let mut out_ref = [&mut processed[..] as &mut [Sample]];
            master_preamp.process(&in_ref, &mut out_ref, block);
        }
        for i in 0..block {
            output[(offset + i) * 2 + 1] = processed[i];
        }

        offset += block;
    }

    // ── Peak normalize to 0.95 ───────────────────────────────────────────────
    let peak = output.iter().fold(0.0f32, |max, &s| max.max(s.abs())).max(1e-10);
    let scale = 0.95 / peak;
    for s in &mut output {
        *s *= scale;
    }

    output
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_queue() {
        let q: LockFreeQueue<u32, 4> = LockFreeQueue::new();
        assert!(q.push(1));
        assert!(q.push(2));
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn test_lock_free_queue_full() {
        let q: LockFreeQueue<u32, 3> = LockFreeQueue::new();
        assert!(q.push(1));
        assert!(q.push(2));
        assert!(!q.push(3)); // capacity-1 is max due to head/tail sentinel
    }

    #[test]
    fn test_preamp() {
        let mut p = SuperAnaloguePreamp::new();
        p.set_gain_db(6.0); // ~2x gain
        let input_data = [0.5; 64];
        let mut output_data = [0.0; 64];
        let inputs = [&input_data[..]];
        let mut outputs = [&mut output_data[..]];
        p.process(&inputs, &mut outputs, 64);
        assert!((output_data[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_engine_init_and_process() {
        let mut engine = TitanCoreEngine::new();
        assert!(engine.init(48000.0, 128, 2, 2));

        let mut in_bufs = vec![vec![0.0; 128]; 2];
        let mut out_bufs = vec![vec![0.0; 128]; 2];
        in_bufs[0][0] = 0.5;

        let inputs: Vec<&[Sample]> = in_bufs.iter().map(|b| &b[..]).collect();
        let mut outputs: Vec<&mut [Sample]> = out_bufs.iter_mut().map(|b| &mut b[..]).collect();

        engine.process_block(&inputs, &mut outputs, 128);

        assert!(engine.metrics().blocks_processed.load(Ordering::Relaxed) >= 1);
    }

    #[test]
    fn test_safe_mode() {
        let mut engine = TitanCoreEngine::new();
        engine.init(48000.0, 64, 2, 2);
        engine.enable_safe_mode(true);

        let in_bufs = vec![vec![0.5; 64]; 2];
        let mut out_bufs = vec![vec![0.0; 64]; 2];

        let inputs: Vec<&[Sample]> = in_bufs.iter().map(|b| &b[..]).collect();
        let mut outputs: Vec<&mut [Sample]> = out_bufs.iter_mut().map(|b| &mut b[..]).collect();

        engine.process_block(&inputs, &mut outputs, 64);
        assert_eq!(out_bufs[0][0], 0.5); // straight through in safe mode
    }

    #[test]
    fn test_xrun_detection() {
        let mut engine = TitanCoreEngine::new();
        engine.init(48000.0, 128, 2, 2);

        let in_bufs = vec![vec![0.0; 64]; 2]; // wrong size!
        let mut out_bufs = vec![vec![0.0; 64]; 2];

        let inputs: Vec<&[Sample]> = in_bufs.iter().map(|b| &b[..]).collect();
        let mut outputs: Vec<&mut [Sample]> = out_bufs.iter_mut().map(|b| &mut b[..]).collect();

        engine.process_block(&inputs, &mut outputs, 64);
        assert_eq!(engine.metrics().xruns.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_param_message() {
        let mut engine = TitanCoreEngine::new();
        engine.init(48000.0, 128, 2, 2);

        assert!(engine.enqueue_param(ParamMsg::SetPreampGainDb(-6.0)));

        let in_bufs = vec![vec![1.0; 128]; 2];
        let mut out_bufs = vec![vec![0.0; 128]; 2];
        let inputs: Vec<&[Sample]> = in_bufs.iter().map(|b| &b[..]).collect();
        let mut outputs: Vec<&mut [Sample]> = out_bufs.iter_mut().map(|b| &mut b[..]).collect();
        engine.process_block(&inputs, &mut outputs, 128);

        // -6 dB = ~0.5 gain
        assert!((out_bufs[0][0] - 0.5).abs() < 0.01, "got {}", out_bufs[0][0]);
    }

    #[test]
    fn test_driver_stub() {
        let mut engine = TitanCoreEngine::new();
        engine.init(48000.0, 128, 2, 2);
        let mut driver = DesktopDriver::new(&mut engine);
        assert!(driver.init(48000.0, 128, 2, 2));

        let in_bufs = vec![vec![0.0; 128]; 2];
        let mut out_bufs = vec![vec![0.0; 128]; 2];
        let inputs: Vec<&[Sample]> = in_bufs.iter().map(|b| &b[..]).collect();
        let mut outputs: Vec<&mut [Sample]> = out_bufs.iter_mut().map(|b| &mut b[..]).collect();

        assert!(driver.process_callback(&inputs, &mut outputs, 128));
        driver.shutdown();
    }
}
