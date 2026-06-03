# Building Red Dirt Rhythm Studio Desktop App

## Prerequisites

1. **Node.js** (v16+ recommended)
2. **Git** (for cloning if needed)
3. **Windows Build Tools** (for native modules if any)

## Installation

1. Clone or copy this repository to your local machine
2. Install dependencies:
   ```bash
   npm install
   ```

## Development

Run the app in development mode:
```bash
npm start
```

## Building for Windows (MSI Installer)

To create Windows MSI installers:

```bash
# Build for both 32-bit and 64-bit Windows
npm run dist

# Or build just for 64-bit
npx electron-builder --win --x64

# Or build just for 32-bit
npx electron-builder --win --ia32
```

The installers will be created in the `dist/` directory.

## Build Configuration

The `package.json` file contains Electron Builder configuration for:
- **Application ID**: `com.reddirt.rhythmsstudio`
- **Output**: MSI installers (both ia32 and x64)
- **Upgrade Code**: `ae261f2d-7c1a-48d2-b133-666f7f5928e1` (for Windows Installer updates)
- **Files Included**: All files except node_modules, dist, build, and documentation
- **Icon**: Place an `icon.ico` file in the `build/` directory for the application icon

## Notes

1. **First Build**: The first build may take longer as Electron downloads binaries
2. **Code Signing**: For production distribution, you'll need to:
   - Obtain a code signing certificate
   - Place it as `build/certificate.pfx`
   - Set the password in environment variable or configure in package.json
3. **Anti-Virus**: New/un-signed installers may trigger false positives with some AV software
4. **Testing**: Always test the installed application on target Windows versions

## Troubleshooting

- **Missing Dependencies**: Ensure you have the latest Node.js and npm
- **Build Failures**: Try clearing cache: `npx electron-builder --win --ia32 --x64 --clean`
- **Performance Issues**: Electron apps have higher memory usage than native apps but provide cross-platform compatibility
- **Audio Issues**: Ensure your system has proper audio drivers and that the browser/Electron has audio permissions

## Alternative Build Methods

If you prefer not to use electron-builder:

### Manual MSI Creation with WiX Toolset
1. Install [WiX Toolset](https://wixtoolset.org/)
2. Create a .wxs file describing your application
3. Compile with `candle.exe` and link with `light.exe`
4. This gives more control but requires more manual work

### Other Packers
- [Inno Setup](https://jrsoftware.org/isinfo.php)
- [Advanced Installer](https://www.advancedinstaller.com/)
- [NSIS](https://nsis.sourceforge.io/Main_Page)

## Post-Build Testing

After building:
1. Install the MSI on a test machine
2. Verify audio input/output works
3. Test recording and playback functionality
4. Check that all UI elements function correctly
5. Verify plugin loading and chaining works
6. Test export/bounce functionality