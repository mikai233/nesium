use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, OnceLock};

static NMI_TRACE_LOG: OnceLock<Option<Mutex<BufWriter<std::fs::File>>>> = OnceLock::new();

#[inline]
pub(crate) fn flag(value: bool) -> u8 {
    u8::from(value)
}

pub(crate) fn log_line(line: &str) {
    let log = NMI_TRACE_LOG.get_or_init(|| {
        let path = std::env::var("NESIUM_NMI_TRACE_PATH").ok()?;
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .ok()
            .map(|f| Mutex::new(BufWriter::with_capacity(256 * 1024, f)))
    });

    if let Some(writer) = log
        && let Ok(mut w) = writer.lock()
    {
        let _ = writeln!(w, "{line}");
        let _ = w.flush();
    }
}
