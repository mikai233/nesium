use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

pub fn pick_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("NES ROM", &["nes", "fds"])
        .pick_file()
}

pub fn save_wav_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("WAV audio", &["wav"])
        .set_file_name("recording.wav")
        .save_file()
}

pub fn write_wav(path: &Path, sample_rate: u32, samples: &[f32]) -> std::io::Result<()> {
    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;

    let frames = (samples.len() / 2) as u32;
    let data_len = frames * bytes_per_sample * channels as u32;
    let riff_len = 36 + data_len;

    let file = File::create(path)?;
    let mut w = BufWriter::new(file);

    // RIFF header
    w.write_all(b"RIFF")?;
    w.write_all(&riff_len.to_le_bytes())?;
    w.write_all(b"WAVE")?;

    // fmt chunk
    w.write_all(b"fmt ")?;
    w.write_all(&16u32.to_le_bytes())?; // PCM chunk size
    w.write_all(&1u16.to_le_bytes())?; // PCM format
    w.write_all(&channels.to_le_bytes())?;
    w.write_all(&sample_rate.to_le_bytes())?;
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample;
    w.write_all(&byte_rate.to_le_bytes())?;
    let block_align = channels * bits_per_sample / 8;
    w.write_all(&block_align.to_le_bytes())?;
    w.write_all(&bits_per_sample.to_le_bytes())?;

    // data chunk
    w.write_all(b"data")?;
    w.write_all(&data_len.to_le_bytes())?;

    for chunk in samples.chunks(2) {
        let l = *chunk.first().unwrap_or(&0.0);
        let r = *chunk.get(1).unwrap_or(&l);
        let to_i16 = |x: f32| -> i16 {
            let v = x.clamp(-1.0, 1.0) * 32767.0;
            v.round() as i16
        };
        let li = to_i16(l);
        let ri = to_i16(r);
        w.write_all(&li.to_le_bytes())?;
        w.write_all(&ri.to_le_bytes())?;
    }

    w.flush()?;
    Ok(())
}
