Native Rust inference port plan

This crate is a scaffold for running a TorchScript-exported version of your voice conversion model natively in Rust using `tch` (libtorch).

Next steps to complete the native port
1. Convert the PyTorch `.pth` model to a TorchScript `.pt` file. Example Python steps (run in a Python environment with the model code available):
   ```python
   import torch
   from so_vits_svc_fork.inference.core import Svc

   svc = Svc(net_g_path='G_0.pth', config_path='config.json', device='cpu')
   # You will need to adapt this to the model's forward signature and tracing inputs
   example_input = torch.randn(1, 16000)  # replace with appropriate shape
   ts = torch.jit.trace(svc.net_g, example_input)
   ts.save('model_ts.pt')
   ```

2. Place `model_ts.pt` next to this crate or pass `--model path/to/model_ts.pt`.
3. Implement preprocessing steps in Rust (resampling, pitch extraction, normalization, etc.) to match the Python reference. Crates to use:
   - `symphonia` for flexible audio decoding
   - `rubato` or `resampler` for resampling
   - `ndarray` for array ops
4. Validate outputs vs Python by running identical inputs and comparing WAVs.

Notes
- The current scaffold assumes the scripted model accepts a single 1xN float tensor and returns a 1xN tensor. Adjust `main.rs` to match the real model signature.
- Converting complex models may require exporting submodules or reworking forward signatures for traceability.
