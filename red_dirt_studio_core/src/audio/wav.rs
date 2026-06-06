use std::fs::File;
use std::path::Path;
use anyhow::Result;
use hound::{WavReader, WavWriter, WavSpec};
use ndarray::Array1;
use super::AudioBuffer;

pub fn read_wav(path: &Path) -> Result<AudioBuffer> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .map(|s| s.unwrap())
            .collect(),
        hound::SampleFormat::Int => reader
            .into_samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect(),
    };
    
    Ok(AudioBuffer {
        data: Array1::from(samples),
        sample_rate: spec.sample_rate,
        channels: spec.channels,
    })
}

pub fn write_wav(path: &Path, buffer: &AudioBuffer) -> Result<()> {
    let spec = WavSpec {
        channels: buffer.channels,
        sample_rate: buffer.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    
    let mut writer = WavWriter::create(path, spec)?;
    for &sample in buffer.data.iter() {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    
    Ok(())
}

pub fn read_flac(path: &Path) -> Result<AudioBuffer> {
    use symphonia::core::io::MediaSourceStream;
    use symphonia::default::get_codecs;
    
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe().format(&Default::default(), mss, &Default::default(), &Default::default())?;
    let mut demux = probed.format;
    
    let track_info = demux.default_track().ok_or_else(|| anyhow::anyhow!("No default track"))?;
    let sample_rate = track_info.codec_params.sample_rate.unwrap_or(48000);
    let channels = track_info.codec_params.channels.unwrap().count() as u16;
    let mut decoder = get_codecs().make(&track_info.codec_params, &Default::default())?;
    
    let mut samples = Vec::new();
    
    while let Ok(packet) = demux.next_packet() {
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                use symphonia::core::audio::SampleBuffer;
                let spec = *audio_buf.spec();
                let frames = audio_buf.frames() as u64;
                let mut sample_buf = SampleBuffer::<f32>::new(frames, spec);
                sample_buf.copy_interleaved_ref(audio_buf);
                samples.extend_from_slice(sample_buf.samples());
            }
            Err(_) => continue,
        }
    }
    
    Ok(AudioBuffer {
        data: Array1::from(samples),
        sample_rate,
        channels,
    })
}

pub fn read_audio_auto(path: &Path) -> Result<AudioBuffer> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "wav" => read_wav(path),
        "flac" => read_flac(path),
        _ => read_wav(path),
    }
}