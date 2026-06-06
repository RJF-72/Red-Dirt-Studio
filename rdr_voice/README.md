Packaging `rdr_voice` for distribution

Strategy
- Build a standalone native inference binary from the existing Python `infer_voice.py` (per-platform) using `PyInstaller` or similar, and ship it alongside the Rust binary. The Rust wrapper will prefer the bundled executable, falling back to invoking `python infer_voice.py` when not present.

Recommended steps (Windows example)
1. Create a virtualenv and install requirements used by `infer_voice.py` (`torch`, `so_vits_svc_fork`, `librosa`, `soundfile`, etc.).
2. Install `pyinstaller`:
   ```bash
   pip install pyinstaller
   ```
3. Build a one-file executable:
   ```bash
   pyinstaller --onefile --name infer_voice infer_voice.py
   ```
4. Copy the produced `dist/infer_voice.exe` next to the Rust binary when packaging your app (e.g., in the installer or zip).

Notes
- You must produce builds per target OS (Windows/Linux/macOS).
- The standalone executable can be large due to bundled Python and dependencies.
- Alternative advanced options: embed Python via `PyO3`, or reimplement inference in Rust with `tch`/libtorch — both are heavier engineering efforts.

Usage
- If the bundled `infer_voice` executable is present next to the Rust binary, the Rust CLI will use it automatically. Otherwise it will attempt to run the local system `python` with `infer_voice.py`.
