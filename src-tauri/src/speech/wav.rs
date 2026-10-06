//! WAV decoding: any channel count and PCM/float format -> mono f32 in [-1, 1].
//! Resampling is left to the engine (sherpa-onnx resamples to the model rate).

use std::path::Path;

use hound::{SampleFormat, WavReader};

use super::error::SpeechError;

pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl DecodedAudio {
    pub fn duration_ms(&self) -> u64 {
        if self.sample_rate == 0 {
            return 0;
        }
        (self.samples.len() as u64 * 1000) / self.sample_rate as u64
    }
}

pub fn read_wav_mono(path: &Path) -> Result<DecodedAudio, SpeechError> {
    let mut reader = WavReader::open(path)
        .map_err(|e| SpeechError::Audio(format!("cannot open {}: {e}", path.display())))?;
    let spec = reader.spec();
    let channels = spec.channels.max(1) as usize;

    let interleaved: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .collect::<Result<_, _>>()
            .map_err(|e| SpeechError::Audio(e.to_string()))?,
        (SampleFormat::Int, bits @ (8 | 16 | 24 | 32)) => {
            let scale = (1u64 << (bits - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale))
                .collect::<Result<_, _>>()
                .map_err(|e| SpeechError::Audio(e.to_string()))?
        }
        (format, bits) => {
            return Err(SpeechError::Audio(format!(
                "unsupported WAV format: {format:?} {bits}-bit"
            )))
        }
    };

    let samples: Vec<f32> = interleaved
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect();

    if samples.is_empty() {
        return Err(SpeechError::Audio("WAV file contains no samples".into()));
    }
    Ok(DecodedAudio {
        samples,
        sample_rate: spec.sample_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_wav(path: &Path, channels: u16, rate: u32, frames: &[(i16, i16)]) {
        let spec = hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for (l, r) in frames {
            w.write_sample(*l).unwrap();
            if channels == 2 {
                w.write_sample(*r).unwrap();
            }
        }
        w.finalize().unwrap();
    }

    #[test]
    fn mono_16bit_is_normalised_and_timed() {
        let p = std::env::temp_dir().join("speechlab_test_mono.wav");
        write_wav(&p, 1, 16000, &vec![(16384, 0); 16000]);
        let a = read_wav_mono(&p).unwrap();
        assert_eq!(a.sample_rate, 16000);
        assert_eq!(a.duration_ms(), 1000);
        assert!((a.samples[0] - 0.5).abs() < 1e-4);
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn stereo_is_mixed_down_by_averaging() {
        let p = std::env::temp_dir().join("speechlab_test_stereo.wav");
        write_wav(&p, 2, 8000, &[(16384, -16384), (16384, 16384)]);
        let a = read_wav_mono(&p).unwrap();
        assert_eq!(a.samples.len(), 2);
        assert!(a.samples[0].abs() < 1e-4);
        assert!((a.samples[1] - 0.5).abs() < 1e-4);
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn missing_file_is_an_audio_error() {
        assert!(matches!(
            read_wav_mono(Path::new("definitely-missing.wav")),
            Err(SpeechError::Audio(_))
        ));
    }
}
