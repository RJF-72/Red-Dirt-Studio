use anyhow::{Context, Result};
use clap::Parser;
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
use std::path::PathBuf;

/// Native inference stub for rdr_voice (loads TorchScript model via libtorch)
#[derive(Parser)]
struct Args {
    #[arg(long)]
    model: PathBuf,
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    output: PathBuf,
}

fn read_wav_mono(path: &PathBuf) -> Result<(Vec<f32>, u32)> {
    let mut reader = WavReader::open(path).with_context(|| format!("opening WAV {:?}", path))?;
    let spec = reader.spec();
    let sr = spec.sample_rate;

    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
        SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap_or(0) as f32 / max)
                .collect()
        }
    };

    // If stereo, downmix by averaging pairs
    let channels = spec.channels as usize;
    let mono = if channels == 1 {
        samples
    } else {
        samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / (chunk.len() as f32))
            .collect()
    };

    Ok((mono, sr))
}

fn write_wav(path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<()> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for &s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("[rdr_voice_native] Loading model: {:?}", args.model);

    // Load TorchScript model
    let module = tch::CModule::load(&args.model).context("loading TorchScript model")?;

    // Read input WAV
    let (audio, sr) = read_wav_mono(&args.input).context("reading input wav")?;
    println!("[rdr_voice_native] Loaded audio: {:.1}s @ {}Hz", audio.len() as f32 / sr as f32, sr);

    // Prepare input tensor: 1 x N
    let input = tch::Tensor::of_slice(&audio).to_kind(tch::Kind::Float).unsqueeze(0);

    // Forward through model (assumes model accepts 1 x N float tensor)
    let output = module
        .forward_is(&[tch::IValue::Tensor(input)])
        .context("running model forward")?;

    // Expect a Tensor output -> convert to Vec<f32>
    let out_tensor = match output {
        tch::IValue::Tensor(t) => t,
        _ => anyhow::bail!("model did not return a tensor"),
    };

    let out_vec: Vec<f32> = Vec::<f32>::from(out_tensor.squeeze());

    write_wav(&args.output, &out_vec, sr).context("writing output wav")?;

    println!("[rdr_voice_native] Done: {:?}", args.output);

    Ok(())
}
