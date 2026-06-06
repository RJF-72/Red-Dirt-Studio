use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

fn main() -> Result<()> {
    // Forward all CLI args to the existing Python inference script/binary.
    let forwarded_args: Vec<String> = env::args().skip(1).collect();

    // Prefer a bundled standalone `infer_voice` executable placed next to this binary.
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    #[cfg(target_os = "windows")]
    let bundled = exe_dir.join("infer_voice.exe");
    #[cfg(not(target_os = "windows"))]
    let bundled = exe_dir.join("infer_voice");

    let mut cmd = if bundled.exists() {
        let mut c = Command::new(bundled);
        c.args(&forwarded_args);
        c
    } else {
        // fallback: run the Python script adjacent to the crate manifest
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("infer_voice.py");
        let python_cmd = "python";
        let mut c = Command::new(python_cmd);
        c.arg(script).args(&forwarded_args);
        c
    };

    cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let status = cmd.status().with_context(|| "failed to execute inference command")?;

    if let Some(code) = status.code() {
        exit(code);
    } else {
        exit(1);
    }
}
