const { app, BrowserWindow, ipcMain, net } = require('electron')
const path = require('path')
const fs = require('fs')
const { spawn } = require('child_process')
const https = require('https')
const http = require('http')

let titanEngineProcess = null
const TITAN_ENGINE_PORT = 5123
const TITAN_ENGINE_URL = `http://localhost:${TITAN_ENGINE_PORT}`

// ── Teleprompter Server Management ──
let teleprompterProcess = null
const TELEPROMPTER_PORT = 8080
const TELEPROMPTER_BIN_RELEASE = path.join(__dirname, '..', 'red_dirt_teleprompter', 'target', 'release', 'rd_teleprompter.exe')
const TELEPROMPTER_BIN_DEBUG = path.join(__dirname, '..', 'red_dirt_teleprompter', 'target', 'debug', 'rd_teleprompter.exe')
const TELEPROMPTER_BIN_PACKAGED = path.join(process.resourcesPath || '', 'teleprompter', 'rd_teleprompter.exe')

function getTeleprompterBin() {
  if (process.resourcesPath && fs.existsSync(TELEPROMPTER_BIN_PACKAGED)) return TELEPROMPTER_BIN_PACKAGED
  if (fs.existsSync(TELEPROMPTER_BIN_RELEASE)) return TELEPROMPTER_BIN_RELEASE
  if (fs.existsSync(TELEPROMPTER_BIN_DEBUG)) return TELEPROMPTER_BIN_DEBUG
  return null
}

function startTeleprompter(event, { lyrics, duration }) {
  if (teleprompterProcess) {
    console.log('[Teleprompter] Already running, restarting...')
    stopTeleprompter()
  }
  const binPath = getTeleprompterBin()
  if (!binPath) {
    console.warn('[Teleprompter] Binary not found')
    if (event) event.sender.send('teleprompter-error', 'Binary not found')
    return
  }
  const tmpFile = path.join(app.getPath('temp'), `rd_teleprompter_${Date.now()}.json`)
  try {
    fs.writeFileSync(tmpFile, JSON.stringify(lyrics), 'utf-8')
  } catch (err) {
    console.error('[Teleprompter] Failed to write lyrics file:', err.message)
    if (event) event.sender.send('teleprompter-error', err.message)
    return
  }
  teleprompterProcess = spawn(binPath, [tmpFile, String(TELEPROMPTER_PORT), String(duration)], {
    cwd: path.dirname(binPath),
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true
  })
  teleprompterProcess.stdout.on('data', (data) => {
    const msg = data.toString().trim()
    console.log(`[Teleprompter] ${msg}`)
    if (msg === 'TELEPROMPTER_READY' && event) {
      event.sender.send('teleprompter-ready', TELEPROMPTER_PORT)
    }
  })
  teleprompterProcess.stderr.on('data', (data) => {
    console.error(`[Teleprompter:err] ${data.toString().trim()}`)
  })
  teleprompterProcess.on('error', (err) => {
    console.error('[Teleprompter] Failed to start:', err.message)
    teleprompterProcess = null
    if (event) event.sender.send('teleprompter-error', err.message)
  })
  teleprompterProcess.on('close', (code) => {
    console.log(`[Teleprompter] Exited with code ${code}`)
    teleprompterProcess = null
  })
  console.log(`[Teleprompter] Starting on port ${TELEPROMPTER_PORT}...`)
}

function stopTeleprompter() {
  if (teleprompterProcess) {
    console.log('[Teleprompter] Shutting down...')
    teleprompterProcess.kill('SIGTERM')
    setTimeout(() => {
      if (teleprompterProcess) {
        teleprompterProcess.kill('SIGKILL')
        teleprompterProcess = null
      }
    }, 3000)
  }
}

// IPC handlers
ipcMain.on('teleprompter-start', (event, data) => startTeleprompter(event, data))
ipcMain.on('teleprompter-stop', () => stopTeleprompter())

// ── Voice Model Management ──
const VOICE_MODELS_DIR = path.join(app.getPath('userData'), 'voice')
const VOICE_DEV_DIR = path.join(__dirname, '..', 'rdr_voice')

const VOICE_MANIFEST = {
  baseUrl: 'https://github.com/red-dirt-rhythm/voice-models/releases/download/v1.0.0',
  files: [
    { name: 'infer_voice.py', size: 0 },
    { name: 'male/logs/44k/config.json', size: 0 },
    { name: 'male/logs/44k/G_0.pth', size: 0 },
    { name: 'male/logs/44k/G_20.pth', size: 0 },
    { name: 'male/logs/44k/G_40.pth', size: 0 },
    { name: 'male/logs/44k/G_60.pth', size: 0 },
    { name: 'female/logs/44k/config.json', size: 0 },
    { name: 'female/logs/44k/G_0.pth', size: 0 }
  ]
}

function getVoiceModelsDir() {
  return VOICE_MODELS_DIR
}

function ensureVoiceDirs() {
  const dirs = [
    VOICE_MODELS_DIR,
    path.join(VOICE_MODELS_DIR, 'male', 'logs', '44k'),
    path.join(VOICE_MODELS_DIR, 'female', 'logs', '44k')
  ]
  for (const dir of dirs) {
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true })
  }
}

function checkVoiceModelsExist() {
  const requiredFiles = VOICE_MANIFEST.files.map(f => path.join(VOICE_MODELS_DIR, f.name))
  return requiredFiles.every(f => fs.existsSync(f))
}

function copyVoiceModelsFromDev(destDir, event) {
  ensureVoiceDirs()
  let copied = 0
  const total = VOICE_MANIFEST.files.length

  for (const file of VOICE_MANIFEST.files) {
    const src = path.join(VOICE_DEV_DIR, file.name)
    const dest = path.join(destDir, file.name)
    if (fs.existsSync(src)) {
      const destPath = path.dirname(dest)
      if (!fs.existsSync(destPath)) fs.mkdirSync(destPath, { recursive: true })
      fs.copyFileSync(src, dest)
    }
    copied++
    if (event) event.sender.send('voice-download-progress', { current: copied, total, file: file.name })
  }
  return checkVoiceModelsExist()
}

function downloadFile(url, dest, event) {
  return new Promise((resolve, reject) => {
    const protocol = url.startsWith('https') ? https : http
    const request = protocol.get(url, { headers: { 'User-Agent': 'RedDirtRhythmStudio' } }, (response) => {
      if (response.statusCode === 301 || response.statusCode === 302) {
        return downloadFile(response.headers.location, dest, event).then(resolve).catch(reject)
      }
      if (response.statusCode !== 200) {
        reject(new Error(`HTTP ${response.statusCode} for ${url}`))
        return
      }
      const totalBytes = parseInt(response.headers['content-length'], 10) || 0
      let downloadedBytes = 0
      const destDir = path.dirname(dest)
      if (!fs.existsSync(destDir)) fs.mkdirSync(destDir, { recursive: true })
      const file = fs.createWriteStream(dest)
      response.pipe(file)
      response.on('data', (chunk) => {
        downloadedBytes += chunk.length
        if (event && totalBytes > 0) {
          event.sender.send('voice-download-file-progress', {
            file: path.basename(dest),
            downloaded: downloadedBytes,
            total: totalBytes
          })
        }
      })
      file.on('finish', () => { file.close(); resolve() })
      file.on('error', (err) => { fs.unlink(dest, () => {}); reject(err) })
    })
    request.on('error', reject)
    request.setTimeout(30000, () => { request.destroy(); reject(new Error('Request timeout')) })
  })
}

async function downloadVoiceModels(event) {
  ensureVoiceDirs()
  let downloaded = 0
  const total = VOICE_MANIFEST.files.length

  for (const file of VOICE_MANIFEST.files) {
    const dest = path.join(VOICE_MODELS_DIR, file.name)
    const url = `${VOICE_MANIFEST.baseUrl}/${file.name}`
    if (event) event.sender.send('voice-download-progress', { current: downloaded, total, file: file.name, status: 'downloading' })
    try {
      await downloadFile(url, dest, event)
    } catch (err) {
      console.error(`[VoiceModels] Failed to download ${file.name}:`, err.message)
      if (event) event.sender.send('voice-download-error', { file: file.name, error: err.message })
    }
    downloaded++
    if (event) event.sender.send('voice-download-progress', { current: downloaded, total, file: file.name, status: 'done' })
  }
  return checkVoiceModelsExist()
}

ipcMain.handle('voice-check', () => {
  return { exists: checkVoiceModelsExist(), dir: VOICE_MODELS_DIR }
})

ipcMain.handle('voice-download', async (event) => {
  if (checkVoiceModelsExist()) return { success: true, dir: VOICE_MODELS_DIR }
  if (fs.existsSync(VOICE_DEV_DIR)) {
    const success = copyVoiceModelsFromDev(VOICE_MODELS_DIR, event)
    return { success, dir: VOICE_MODELS_DIR }
  }
  const success = await downloadVoiceModels(event)
  return { success, dir: VOICE_MODELS_DIR }
})

// ── TitanEngine V10 Server Management ──
function startTitanEngine() {
  const engineScript = path.join(__dirname, '..', 'titanengine_extract', 'TitanEngine', 'WORKING_ENGINE.py')
  if (!fs.existsSync(engineScript)) {
    console.warn('TitanEngine V10 server script not found at:', engineScript)
    return
  }

  titanEngineProcess = spawn('python', [engineScript, '--port', String(TITAN_ENGINE_PORT)], {
    cwd: path.dirname(engineScript),
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, FLASK_PORT: String(TITAN_ENGINE_PORT) }
  })

  titanEngineProcess.stdout.on('data', (data) => {
    console.log(`[TitanEngine] ${data.toString().trim()}`)
  })

  titanEngineProcess.stderr.on('data', (data) => {
    console.error(`[TitanEngine:err] ${data.toString().trim()}`)
  })

  titanEngineProcess.on('error', (err) => {
    console.error('[TitanEngine] Failed to start:', err.message)
    titanEngineProcess = null
  })

  titanEngineProcess.on('close', (code) => {
    console.log(`[TitanEngine] Process exited with code ${code}`)
    titanEngineProcess = null
  })

  console.log(`[TitanEngine] Starting server on port ${TITAN_ENGINE_PORT}...`)
}

function stopTitanEngine() {
  if (titanEngineProcess) {
    console.log('[TitanEngine] Shutting down...')
    titanEngineProcess.kill('SIGTERM')
    setTimeout(() => {
      if (titanEngineProcess) {
        titanEngineProcess.kill('SIGKILL')
        titanEngineProcess = null
      }
    }, 3000)
  }
}

function createWindow () {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  })

  // Expose URLs to the renderer
  process.env.TITAN_ENGINE_URL = TITAN_ENGINE_URL
  process.env.TELEPROMPTER_PORT = String(TELEPROMPTER_PORT)
  process.env.VOICE_MODELS_DIR = VOICE_MODELS_DIR

  win.loadFile('index.html')

  if (process.env.NODE_ENV === 'development') {
    win.webContents.openDevTools()
  }
}

app.whenReady().then(() => {
  startTitanEngine()
  createWindow()

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') {
    stopTitanEngine()
    stopTeleprompter()
    app.quit()
  }
})

app.on('before-quit', () => {
  stopTitanEngine()
  stopTeleprompter()
})

app.on('will-quit', () => {
  stopTitanEngine()
  stopTeleprompter()
})