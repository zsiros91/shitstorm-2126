use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::log::BoxedLayer;
use bevy::log::tracing_subscriber::fmt;
use bevy::prelude::*;

/// Where the diagnostic log is written. Release builds on Windows run without
/// a console, so this file is the only way to see adapter, asset and rendering
/// errors there. Every platform uses the same location for consistency.
pub fn log_path() -> PathBuf {
    std::env::temp_dir().join("shitstorm-2126.log")
}

#[derive(Clone)]
struct SharedFile(Arc<Mutex<File>>);

impl Write for SharedFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("log lock poisoned"))?
            .write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("log lock poisoned"))?
            .flush()
    }
}

/// Extra tracing layer that mirrors every log line into [`log_path`].
pub fn file_layer(_app: &mut App) -> Option<BoxedLayer> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .ok()?;
    let shared = SharedFile(Arc::new(Mutex::new(file)));
    let layer = fmt::layer()
        .with_ansi(false)
        .with_writer(move || shared.clone());
    Some(Box::new(layer))
}

/// Record panics in the same file so a crash still leaves a trace.
pub fn install_panic_hook() {
    let path = log_path();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "panic: {info}");
        }
        default_hook(info);
    }));
}
