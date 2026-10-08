//! Engine-independent chunking of long audio (D-035).
//!
//! Some engines handle long recordings badly (dropped ending, repetition loops, see I-017, I-018,
//! I-019, I-029). The idea: cut the audio at silences into pieces of at most about 25 s,
//! transcribe each piece with the SAME provider, and join the texts.
//!
//! This module never knows an engine. It has three parts:
//! - `plan_segments`: pure planning from speech spans (where the speaker talks) to cut segments;
//! - `energy_spans`: a dependency-free speech detector, used by the tests and as a fallback;
//!   the real detector is the Silero VAD in `vad.rs`;
//! - `transcribe_chunked`: cuts a WAV into temporary WAV pieces, calls any provider on each
//!   piece and sums the timings.
//!
//! A clip that already fits in one segment is NOT touched: the provider receives the original
//! file, so its output is identical to the whole-clip path.

use std::path::{Path, PathBuf};
use std::time::Instant;

use super::error::SpeechError;
use super::provider::{CancelToken, SpeechToTextProvider};
use super::types::{TranscribeRequest, TranscribeResult};
use super::wav;

/// Sample rate used for detection and for the cut pieces.
pub const CHUNK_RATE: u32 = 16_000;

/// A half-open range of samples `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Longest segment handed to an engine, padding included.
    pub max_segment_s: f32,
    /// Audio kept on each side of the speech inside a segment.
    pub padding_s: f32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self { max_segment_s: 25.0, padding_s: 0.2 }
    }
}

impl ChunkConfig {
    fn max_samples(&self) -> usize {
        (self.max_segment_s * CHUNK_RATE as f32) as usize
    }
    fn pad_samples(&self) -> usize {
        (self.padding_s * CHUNK_RATE as f32) as usize
    }
}

// ---------------------------------------------------------------- planning

/// Frame length and hop (in samples) of the energy analysis: 20 ms frames every 10 ms.
const FRAME: usize = (CHUNK_RATE as usize) / 50;
const HOP: usize = FRAME / 2;

fn frame_rms(samples: &[f32], start: usize) -> f32 {
    let end = (start + FRAME).min(samples.len());
    let seg = &samples[start.min(end)..end];
    if seg.is_empty() {
        return 0.0;
    }
    (seg.iter().map(|s| s * s).sum::<f32>() / seg.len() as f32).sqrt()
}

/// Sample index of the quietest point in `[lo, hi)`: the centre of the lowest-energy run of
/// frames. Used only when speech is continuous for longer than a segment may be.
pub fn quietest_point(samples: &[f32], lo: usize, hi: usize) -> usize {
    let hi = hi.min(samples.len());
    if hi <= lo + FRAME {
        return (lo + hi) / 2;
    }
    let starts: Vec<usize> = (lo..hi - FRAME + 1).step_by(HOP).collect();
    let energies: Vec<f32> = starts.iter().map(|&s| frame_rms(samples, s)).collect();
    let min = energies.iter().copied().fold(f32::INFINITY, f32::min);
    let tolerance = min * 1.05 + 1e-7;
    let lowest: Vec<usize> = (0..energies.len()).filter(|&i| energies[i] <= tolerance).collect();
    let middle = lowest[lowest.len() / 2];
    starts[middle] + FRAME / 2
}

fn tidy(spans: &[Span], n: usize) -> Vec<Span> {
    let mut v: Vec<Span> = spans
        .iter()
        .map(|s| Span { start: s.start.min(n), end: s.end.min(n) })
        .filter(|s| !s.is_empty())
        .collect();
    v.sort_by_key(|s| s.start);
    let mut out: Vec<Span> = Vec::new();
    for s in v {
        match out.last_mut() {
            Some(last) if s.start <= last.end => last.end = last.end.max(s.end),
            _ => out.push(s),
        }
    }
    out
}

/// Turns speech spans into the segments to transcribe.
///
/// - empty audio gives no segment;
/// - audio that fits in one segment gives exactly one segment: the whole clip, silence included
///   (so the engine output is the same as without chunking);
/// - otherwise speech is grouped greedily into segments of at most `max_segment_s`, cut in the
///   middle of the pause between two spans, with `padding_s` kept on both sides of the speech;
/// - a speech span longer than a segment may be is cut at the quietest point of its second half;
/// - audio without any speech gives no segment.
pub fn plan_segments(samples: &[f32], spans: &[Span], cfg: &ChunkConfig) -> Vec<Span> {
    let n = samples.len();
    if n == 0 {
        return Vec::new();
    }
    let max = cfg.max_samples().max(2 * FRAME);
    if n <= max {
        return vec![Span { start: 0, end: n }];
    }
    let pad = cfg.pad_samples().min(max / 4);
    let budget = max - 2 * pad; // speech that fits in one segment, padding excluded

    // 1. Speech longer than the budget is cut at its quietest point (second half of the budget).
    let mut pieces: Vec<Span> = Vec::new();
    for span in tidy(spans, n) {
        let mut start = span.start;
        while span.end - start > budget {
            let cut = quietest_point(samples, start + budget / 2, start + budget);
            pieces.push(Span { start, end: cut });
            start = cut;
        }
        pieces.push(Span { start, end: span.end });
    }
    if pieces.is_empty() {
        return Vec::new();
    }

    // 2. Greedy grouping of pieces into segments that stay within the limit.
    let mut groups: Vec<Span> = Vec::new();
    for p in pieces {
        match groups.last_mut() {
            Some(g) if p.end - g.start + 2 * pad <= max => g.end = p.end,
            _ => groups.push(p),
        }
    }

    // 3. Boundaries: padding on both sides, but never past the middle of the pause.
    let mut out = Vec::with_capacity(groups.len());
    for (i, g) in groups.iter().enumerate() {
        let start = match i.checked_sub(1).map(|j| groups[j]) {
            Some(prev) => (g.start.saturating_sub(pad)).max((prev.end + g.start) / 2),
            None => g.start.saturating_sub(pad),
        };
        let end = match groups.get(i + 1) {
            Some(next) => (g.end + pad).min((g.end + next.start) / 2),
            None => (g.end + pad).min(n),
        };
        out.push(Span { start, end });
    }
    out
}

// ---------------------------------------------------------------- energy detector

/// Dependency-free speech detector based on frame energy. Adequate for clean synthetic signals
/// (tests) and as a fallback; real speech is detected by the Silero VAD (`vad.rs`).
/// Pauses shorter than `min_silence_s` are bridged; bursts shorter than `min_speech_s` are dropped.
pub fn energy_spans(samples: &[f32], min_silence_s: f32, min_speech_s: f32) -> Vec<Span> {
    if samples.len() < FRAME {
        return Vec::new();
    }
    let starts: Vec<usize> = (0..=samples.len() - FRAME).step_by(HOP).collect();
    let energies: Vec<f32> = starts.iter().map(|&s| frame_rms(samples, s)).collect();
    let mut sorted = energies.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let noise = sorted[sorted.len() / 10];
    let loud = sorted[sorted.len() * 9 / 10];
    // Silence is clearly below speech: three times the noise floor, but never above 30 % of the
    // loud level (continuous speech has no quiet frames), and never below 5 % of it.
    let threshold = (noise * 3.0).min(loud * 0.3).max(loud * 0.05).max(0.003);

    let hop_s = HOP as f32 / CHUNK_RATE as f32;
    let max_gap = (min_silence_s / hop_s).ceil() as usize;
    let mut spans: Vec<(usize, usize)> = Vec::new(); // frame indices, inclusive
    for (i, &e) in energies.iter().enumerate() {
        if e < threshold {
            continue;
        }
        match spans.last_mut() {
            Some(last) if i - last.1 <= max_gap => last.1 = i,
            _ => spans.push((i, i)),
        }
    }
    let min_frames = (min_speech_s / hop_s).ceil() as usize;
    spans
        .into_iter()
        .filter(|(a, b)| b - a + 1 >= min_frames)
        .map(|(a, b)| Span { start: starts[a], end: (starts[b] + FRAME).min(samples.len()) })
        .collect()
}

// ---------------------------------------------------------------- resampling and WAV pieces

/// Linear-interpolation resampling to `CHUNK_RATE`. Good enough for voice detection and for
/// feeding engines that resample again themselves; only used when the clip is not at 16 kHz.
pub fn to_chunk_rate(samples: &[f32], rate: u32) -> Vec<f32> {
    if rate == CHUNK_RATE || rate == 0 || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = rate as f64 / CHUNK_RATE as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let a = pos.floor() as usize;
            let b = (a + 1).min(samples.len() - 1);
            let t = (pos - a as f64) as f32;
            samples[a.min(samples.len() - 1)] * (1.0 - t) + samples[b] * t
        })
        .collect()
}

fn write_piece(path: &Path, samples: &[f32]) -> Result<(), SpeechError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: CHUNK_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let io = |e: hound::Error| SpeechError::Audio(format!("cannot write {}: {e}", path.display()));
    let mut w = hound::WavWriter::create(path, spec).map_err(io)?;
    for s in samples {
        w.write_sample((s.clamp(-1.0, 1.0) * 32767.0).round() as i16).map_err(io)?;
    }
    w.finalize().map_err(io)
}

/// Temporary folder of cut pieces, removed when dropped (also after an error or a cancellation).
struct PieceDir(PathBuf);

impl PieceDir {
    fn create() -> Result<Self, SpeechError> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("speechlab_chunks_{}_{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| SpeechError::Audio(format!("cannot create {}: {e}", dir.display())))?;
        Ok(Self(dir))
    }
}

impl Drop for PieceDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---------------------------------------------------------------- chunked transcription

/// What the chunker did, recorded next to the result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkReport {
    /// 1 when the clip was passed through unchanged.
    pub segments: usize,
    /// Time spent detecting speech, planning and writing the pieces (not counted as inference).
    pub chunking_ms: u64,
    /// Start and end of every segment in the clip, in milliseconds.
    pub bounds_ms: Vec<(u64, u64)>,
}

/// Finds where the speech is in a 16 kHz mono signal.
pub type SpanDetector<'a> = &'a dyn Fn(&[f32]) -> Result<Vec<Span>, SpeechError>;

/// Joins the texts of the pieces with single spaces, skipping empty ones.
pub fn join_texts(texts: &[String]) -> String {
    texts.iter().map(|t| t.trim()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(" ")
}

/// Transcribes `request.audio_path` through any provider, cutting it at silences when it is
/// longer than one segment. Timings are summed over the pieces; the detection time is reported
/// separately in `ChunkReport::chunking_ms`.
pub fn transcribe_chunked(
    provider: &dyn SpeechToTextProvider,
    request: &TranscribeRequest,
    cancel: &CancelToken,
    detect: SpanDetector,
    cfg: &ChunkConfig,
) -> Result<(TranscribeResult, ChunkReport), SpeechError> {
    let started = Instant::now();
    let audio = wav::read_wav_mono(Path::new(&request.audio_path))?;
    let audio_ms = audio.duration_ms();
    if (audio.samples.len() as u64 * CHUNK_RATE as u64) / audio.sample_rate.max(1) as u64 <= cfg.max_samples() as u64 {
        let result = provider.transcribe(request, cancel)?;
        return Ok((result, ChunkReport { segments: 1, chunking_ms: 0, bounds_ms: vec![(0, audio_ms)] }));
    }

    let samples = to_chunk_rate(&audio.samples, audio.sample_rate);
    let segments = plan_segments(&samples, &detect(&samples)?, cfg);
    let pieces = PieceDir::create()?;
    let mut files = Vec::with_capacity(segments.len());
    for (i, seg) in segments.iter().enumerate() {
        let path = pieces.0.join(format!("piece-{i:03}.wav"));
        write_piece(&path, &samples[seg.start..seg.end])?;
        files.push(path);
    }
    let chunking_ms = started.elapsed().as_millis() as u64;
    let to_ms = |s: usize| s as u64 * 1000 / CHUNK_RATE as u64;
    let report = ChunkReport {
        segments: segments.len(),
        chunking_ms,
        bounds_ms: segments.iter().map(|s| (to_ms(s.start), to_ms(s.end))).collect(),
    };

    let mut texts = Vec::new();
    let mut total: Option<TranscribeResult> = None;
    for path in &files {
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }
        let piece_request = TranscribeRequest { audio_path: path.display().to_string(), ..request.clone() };
        let r = provider.transcribe(&piece_request, cancel)?;
        texts.push(r.text.clone());
        total = Some(match total {
            None => r,
            Some(mut t) => {
                t.processing_ms += r.processing_ms;
                t.load_ms += r.load_ms;
                t.cold_start |= r.cold_start;
                t.cpu_ms = match (t.cpu_ms, r.cpu_ms) {
                    (Some(a), Some(b)) => Some(a + b),
                    (a, b) => a.or(b),
                };
                t.peak_memory_mb = match (t.peak_memory_mb, r.peak_memory_mb) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
                t
            }
        });
    }

    // Audio without any speech: nothing was sent to the engine, the text is empty.
    let mut result = total.unwrap_or(TranscribeResult {
        provider_id: request.provider_id.clone(),
        model_id: request.model_id.clone(),
        language: request.language.clone(),
        text: String::new(),
        processing_ms: 0,
        load_ms: 0,
        cold_start: false,
        audio_ms: None,
        rtf: None,
        threads: 0,
        peak_memory_mb: None,
        cpu_ms: None,
        decoding: "none (no speech detected)".into(),
        is_mock: false,
    });
    result.text = join_texts(&texts);
    // The clock and the real-time factor refer to the whole clip, not to a piece.
    result.audio_ms = Some(audio_ms);
    result.rtf = (audio_ms > 0).then(|| result.processing_ms as f64 / audio_ms as f64);
    Ok((result, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    const R: usize = CHUNK_RATE as usize;

    // Synthetic signals only: tones stand for speech, zeros for silence.
    fn tone(seconds: f32, amp: f32) -> Vec<f32> {
        (0..(seconds * R as f32) as usize)
            .map(|i| amp * (2.0 * std::f32::consts::PI * 220.0 * i as f32 / R as f32).sin())
            .collect()
    }
    fn silence(seconds: f32) -> Vec<f32> {
        vec![0.0; (seconds * R as f32) as usize]
    }
    fn cat(parts: &[Vec<f32>]) -> Vec<f32> {
        parts.concat()
    }
    fn secs(samples: usize) -> f32 {
        samples as f32 / R as f32
    }
    fn plan(samples: &[f32]) -> Vec<Span> {
        plan_segments(samples, &energy_spans(samples, 0.3, 0.2), &ChunkConfig::default())
    }
    fn max_len_ok(segs: &[Span]) -> bool {
        segs.iter().all(|s| secs(s.len()) <= 25.0 + 1e-3)
    }

    #[test]
    fn short_clip_stays_one_whole_segment() {
        let a = cat(&[silence(0.5), tone(4.0, 0.3), silence(1.0), tone(3.0, 0.3), silence(0.5)]);
        assert_eq!(plan(&a), vec![Span { start: 0, end: a.len() }]);
        // Even a clip of silence: the whole clip, so that the engine sees what it saw before.
        let s = silence(5.0);
        assert_eq!(plan(&s), vec![Span { start: 0, end: s.len() }]);
    }

    #[test]
    fn long_clip_is_split_in_the_pauses_and_loses_no_speech() {
        let a = cat(&[silence(0.5), tone(10.0, 0.3), silence(1.0), tone(10.0, 0.3), silence(1.0), tone(10.0, 0.3), silence(0.5)]);
        assert!(secs(a.len()) > 25.0);
        let segs = plan(&a);
        assert_eq!(segs.len(), 2, "two bursts fit together, the third goes alone: {segs:?}");
        assert!(max_len_ok(&segs));
        assert!(segs[0].end <= segs[1].start, "segments never overlap");
        // The cut is inside the pause between the second and the third burst: pure silence.
        let cut = segs[0].end;
        assert!(a[cut.saturating_sub(100)..cut + 100].iter().all(|s| s.abs() < 1e-6), "cut inside speech");
        // No speech sample is left outside the segments.
        let covered: Vec<bool> = {
            let mut c = vec![false; a.len()];
            for s in &segs {
                c[s.start..s.end].iter_mut().for_each(|x| *x = true);
            }
            c
        };
        assert!(a.iter().zip(&covered).all(|(s, c)| *c || s.abs() < 1e-6), "speech was dropped");
    }

    #[test]
    fn continuous_speech_is_cut_at_the_quietest_point() {
        // 40 s of tone without any pause, except a barely quieter 100 ms patch at 20 s.
        let mut a = tone(40.0, 0.3);
        for s in &mut a[20 * R..20 * R + R / 10] {
            *s *= 0.02;
        }
        let spans = energy_spans(&a, 0.3, 0.2);
        assert_eq!(spans.len(), 1, "the dip is shorter than a pause");
        let segs = plan_segments(&a, &spans, &ChunkConfig::default());
        assert_eq!(segs.len(), 2);
        assert!(max_len_ok(&segs));
        let cut = secs(segs[0].end);
        assert!((cut - 20.05).abs() < 0.15, "cut at {cut} s, expected the quiet patch near 20 s");
        assert_eq!(segs[0].end, segs[1].start, "continuous speech: the two segments touch");
    }

    #[test]
    fn very_long_continuous_speech_never_exceeds_the_limit() {
        let a = tone(70.0, 0.3);
        let segs = plan(&a);
        assert!(segs.len() >= 3);
        assert!(max_len_ok(&segs), "{segs:?}");
        assert_eq!(segs[0].start, 0);
        assert_eq!(segs.last().unwrap().end, a.len());
        assert!(segs.windows(2).all(|w| w[0].end == w[1].start));
    }

    #[test]
    fn padding_is_clamped_to_the_clip_and_to_the_middle_of_a_short_pause() {
        let a = cat(&[tone(14.0, 0.3), silence(0.2), tone(14.0, 0.3)]); // speech at both edges
        let segs = plan_segments(&a, &energy_spans(&a, 0.1, 0.2), &ChunkConfig::default());
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].start, 0);
        assert_eq!(segs[1].end, a.len());
        // A 0.2 s pause is shorter than twice the padding: the cut is its middle, no overlap.
        assert_eq!(segs[0].end, segs[1].start);
        let cut = secs(segs[0].end);
        assert!((cut - 14.1).abs() < 0.1, "cut at {cut}");
    }

    #[test]
    fn empty_and_silent_inputs() {
        assert!(plan(&[]).is_empty());
        assert!(plan(&silence(60.0)).is_empty(), "a long silence has nothing to transcribe");
        assert!(energy_spans(&silence(10.0), 0.3, 0.2).is_empty());
        assert!(energy_spans(&[0.1; 10], 0.3, 0.2).is_empty());
    }

    #[test]
    fn spans_outside_the_clip_or_overlapping_are_tidied() {
        let a = cat(&[tone(30.0, 0.3)]);
        let spans = [Span { start: 100_000, end: 90_000 }, Span { start: 0, end: 300_000 }, Span { start: 200_000, end: 9_999_999 }];
        let segs = plan_segments(&a, &spans, &ChunkConfig::default());
        assert!(max_len_ok(&segs) && segs.last().unwrap().end <= a.len());
    }

    #[test]
    fn resampling_changes_the_length_in_proportion() {
        let a = vec![0.5; 11_025];
        let b = to_chunk_rate(&a, 11_025);
        assert!((b.len() as i64 - 16_000).abs() <= 1);
        assert!(b.iter().all(|s| (s - 0.5).abs() < 1e-6));
        assert_eq!(to_chunk_rate(&a, CHUNK_RATE).len(), a.len());
    }

    #[test]
    fn texts_are_joined_with_single_spaces() {
        assert_eq!(join_texts(&["Un.".into(), " ".into(), " Deux. ".into(), "".into(), "Trois.".into()]), "Un. Deux. Trois.");
        assert_eq!(join_texts(&[]), "");
    }

    /// Records the length of every file it receives and answers "pieceN".
    struct Recorder {
        seen: Mutex<Vec<(String, f32)>>,
    }

    impl SpeechToTextProvider for Recorder {
        fn info(&self) -> super::super::types::ProviderInfo {
            unreachable!()
        }
        fn transcribe(&self, request: &TranscribeRequest, _: &CancelToken) -> Result<TranscribeResult, SpeechError> {
            let audio = wav::read_wav_mono(Path::new(&request.audio_path))?;
            let mut seen = self.seen.lock().unwrap();
            seen.push((request.audio_path.clone(), audio.samples.len() as f32 / audio.sample_rate as f32));
            Ok(TranscribeResult {
                provider_id: "rec".into(),
                model_id: "rec".into(),
                language: request.language.clone(),
                text: format!("piece{}", seen.len()),
                processing_ms: 100,
                load_ms: if seen.len() == 1 { 40 } else { 0 },
                cold_start: seen.len() == 1,
                audio_ms: Some(1),
                rtf: Some(9.0),
                threads: 4,
                peak_memory_mb: Some(100.0 + seen.len() as f64),
                cpu_ms: Some(10),
                decoding: "test".into(),
                is_mock: false,
            })
        }
    }

    fn write_clip(name: &str, samples: &[f32], rate: u32) -> String {
        let p = std::env::temp_dir().join(format!("speechlab_chunk_test_{name}.wav"));
        let spec = hound::WavSpec { channels: 1, sample_rate: rate, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut w = hound::WavWriter::create(&p, spec).unwrap();
        for s in samples {
            w.write_sample((s * 32767.0) as i16).unwrap();
        }
        w.finalize().unwrap();
        p.display().to_string()
    }

    fn request(path: &str) -> TranscribeRequest {
        TranscribeRequest { provider_id: "rec".into(), model_id: "rec".into(), language: "fr".into(), audio_path: path.into() }
    }

    #[test]
    fn short_clip_reaches_the_provider_untouched() {
        let path = write_clip("short", &cat(&[tone(3.0, 0.3), silence(1.0)]), CHUNK_RATE);
        let rec = Recorder { seen: Mutex::new(Vec::new()) };
        let detect = |_: &[f32]| -> Result<Vec<Span>, SpeechError> { panic!("the detector must not run on a short clip") };
        let (r, report) = transcribe_chunked(&rec, &request(&path), &CancelToken::new(), &detect, &ChunkConfig::default()).unwrap();
        assert_eq!(report.segments, 1);
        assert_eq!(report.chunking_ms, 0);
        assert_eq!(r.text, "piece1");
        let seen = rec.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, path, "the original file is passed, not a copy");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn long_clip_is_cut_transcribed_piece_by_piece_and_timings_are_summed() {
        let clip = cat(&[tone(10.0, 0.3), silence(1.0), tone(10.0, 0.3), silence(1.0), tone(10.0, 0.3)]);
        let path = write_clip("long", &clip, CHUNK_RATE);
        let rec = Recorder { seen: Mutex::new(Vec::new()) };
        let detect = |s: &[f32]| -> Result<Vec<Span>, SpeechError> { Ok(energy_spans(s, 0.3, 0.2)) };
        let (r, report) = transcribe_chunked(&rec, &request(&path), &CancelToken::new(), &detect, &ChunkConfig::default()).unwrap();
        assert_eq!(report.segments, 2);
        assert_eq!(report.bounds_ms.len(), 2);
        assert_eq!(r.text, "piece1 piece2");
        assert_eq!(r.processing_ms, 200, "inference times are summed");
        assert_eq!(r.load_ms, 40);
        assert!(r.cold_start);
        assert_eq!(r.cpu_ms, Some(20));
        assert_eq!(r.peak_memory_mb, Some(102.0), "peak memory is the maximum, not the sum");
        assert_eq!(r.audio_ms, Some(32_000));
        assert!((r.rtf.unwrap() - 200.0 / 32_000.0).abs() < 1e-9, "RTF refers to the whole clip");
        let seen = rec.seen.lock().unwrap();
        assert!(seen.iter().all(|(p, secs)| p != &path && *secs <= 25.0));
        assert!(seen.iter().all(|(p, _)| !Path::new(p).exists()), "temporary pieces are removed");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn non_16k_clip_is_resampled_before_cutting() {
        let clip = vec![0.2; 11_025 * 30];
        let path = write_clip("rate11k", &clip, 11_025);
        let rec = Recorder { seen: Mutex::new(Vec::new()) };
        let detect = |s: &[f32]| -> Result<Vec<Span>, SpeechError> { Ok(vec![Span { start: 0, end: s.len() }]) };
        let (_, report) = transcribe_chunked(&rec, &request(&path), &CancelToken::new(), &detect, &ChunkConfig::default()).unwrap();
        assert_eq!(report.segments, 2);
        let seen = rec.seen.lock().unwrap();
        assert!(seen.iter().all(|(_, secs)| *secs <= 25.0));
        assert!((seen.iter().map(|(_, s)| s).sum::<f32>() - 30.0).abs() < 0.1, "no audio lost or duplicated");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn long_silence_sends_nothing_to_the_engine() {
        let path = write_clip("silent", &silence(40.0), CHUNK_RATE);
        let rec = Recorder { seen: Mutex::new(Vec::new()) };
        let detect = |s: &[f32]| -> Result<Vec<Span>, SpeechError> { Ok(energy_spans(s, 0.3, 0.2)) };
        let (r, report) = transcribe_chunked(&rec, &request(&path), &CancelToken::new(), &detect, &ChunkConfig::default()).unwrap();
        assert_eq!(report.segments, 0);
        assert_eq!(r.text, "");
        assert!(rec.seen.lock().unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cancellation_stops_between_pieces() {
        let clip = cat(&[tone(14.0, 0.3), silence(1.0), tone(14.0, 0.3)]);
        let path = write_clip("cancel", &clip, CHUNK_RATE);
        let rec = Recorder { seen: Mutex::new(Vec::new()) };
        let cancel = CancelToken::new();
        cancel.cancel();
        let detect = |s: &[f32]| -> Result<Vec<Span>, SpeechError> { Ok(energy_spans(s, 0.3, 0.2)) };
        let r = transcribe_chunked(&rec, &request(&path), &cancel, &detect, &ChunkConfig::default());
        assert_eq!(r.err(), Some(SpeechError::Cancelled));
        let _ = std::fs::remove_file(path);
    }
}
