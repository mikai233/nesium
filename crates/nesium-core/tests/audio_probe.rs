mod common;

use anyhow::{Context, Result, bail};
use common::{
    run_rom_audio_pcm16_for_frame_range_with_rate,
    run_rom_audio_pcm16_sha1_for_frame_range_with_rate,
};

#[test]
#[ignore = "manual audio hash probe"]
fn audio_hash_probe() -> Result<()> {
    let rom = std::env::var("NESIUM_AUDIO_PROBE_ROM")
        .context("missing NESIUM_AUDIO_PROBE_ROM (absolute path or vendor-relative path)")?;

    let start_frame = std::env::var("NESIUM_AUDIO_PROBE_START")
        .ok()
        .map(|v| {
            v.parse::<usize>()
                .with_context(|| format!("invalid NESIUM_AUDIO_PROBE_START '{v}'"))
        })
        .transpose()?
        .unwrap_or(0);

    let end_frame = std::env::var("NESIUM_AUDIO_PROBE_END")
        .ok()
        .map(|v| {
            v.parse::<usize>()
                .with_context(|| format!("invalid NESIUM_AUDIO_PROBE_END '{v}'"))
        })
        .transpose()?
        .unwrap_or(600);
    let sample_rate = std::env::var("NESIUM_AUDIO_PROBE_SAMPLE_RATE")
        .ok()
        .map(|v| {
            v.parse::<u32>()
                .with_context(|| format!("invalid NESIUM_AUDIO_PROBE_SAMPLE_RATE '{v}'"))
        })
        .transpose()?
        .unwrap_or(48_000);

    if start_frame >= end_frame {
        bail!(
            "invalid audio range: start_frame {} must be < end_frame {}",
            start_frame,
            end_frame
        );
    }

    let (hash, sample_count) = run_rom_audio_pcm16_sha1_for_frame_range_with_rate(
        &rom,
        start_frame,
        end_frame,
        sample_rate,
    )?;

    println!("[audio-probe] rom={rom}");
    println!("[audio-probe] start_frame={start_frame} end_frame={end_frame}");
    println!("[audio-probe] sample_rate={sample_rate}");
    println!("[audio-probe] hash={hash} sample_count={sample_count}");
    Ok(())
}

#[test]
#[ignore = "manual audio raw dump probe"]
fn audio_raw_dump_probe() -> Result<()> {
    let rom = std::env::var("NESIUM_AUDIO_PROBE_ROM")
        .context("missing NESIUM_AUDIO_PROBE_ROM (absolute path or vendor-relative path)")?;
    let out_raw = std::env::var("NESIUM_AUDIO_PROBE_RAW_OUT")
        .context("missing NESIUM_AUDIO_PROBE_RAW_OUT")?;

    let start_frame = std::env::var("NESIUM_AUDIO_PROBE_START")
        .ok()
        .map(|v| {
            v.parse::<usize>()
                .with_context(|| format!("invalid NESIUM_AUDIO_PROBE_START '{v}'"))
        })
        .transpose()?
        .unwrap_or(0);

    let end_frame = std::env::var("NESIUM_AUDIO_PROBE_END")
        .ok()
        .map(|v| {
            v.parse::<usize>()
                .with_context(|| format!("invalid NESIUM_AUDIO_PROBE_END '{v}'"))
        })
        .transpose()?
        .unwrap_or(600);
    let sample_rate = std::env::var("NESIUM_AUDIO_PROBE_SAMPLE_RATE")
        .ok()
        .map(|v| {
            v.parse::<u32>()
                .with_context(|| format!("invalid NESIUM_AUDIO_PROBE_SAMPLE_RATE '{v}'"))
        })
        .transpose()?
        .unwrap_or(48_000);

    if start_frame >= end_frame {
        bail!(
            "invalid audio range: start_frame {} must be < end_frame {}",
            start_frame,
            end_frame
        );
    }

    let pcm =
        run_rom_audio_pcm16_for_frame_range_with_rate(&rom, start_frame, end_frame, sample_rate)?;
    let mut out = Vec::with_capacity(pcm.len() * 2);
    for s in &pcm {
        out.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(&out_raw, &out).with_context(|| format!("writing raw audio to {}", out_raw))?;

    println!("[audio-probe-raw] rom={rom}");
    println!("[audio-probe-raw] start_frame={start_frame} end_frame={end_frame}");
    println!("[audio-probe-raw] sample_rate={sample_rate}");
    println!(
        "[audio-probe-raw] raw_out={} sample_count={}",
        out_raw,
        pcm.len()
    );
    Ok(())
}
