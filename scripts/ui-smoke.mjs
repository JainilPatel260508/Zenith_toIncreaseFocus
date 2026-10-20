// This explicit development command is never started by or bundled with the desktop app.
import { createServer } from 'vite'
import react from '@vitejs/plugin-react'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { resolve, join } from 'node:path'
import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'

const dir = mkdtempSync(join(tmpdir(), 'zenith-ui-'))
const db = join(dir, 'verification.sqlite3')
const child = spawn(
  process.execPath,
  [
    'scripts/cargo.mjs',
    'run',
    '--locked',
    '--manifest-path',
    'src-tauri/Cargo.toml',
    '--features',
    'test-support',
    '--example',
    'ui_bridge',
    '--',
    db,
  ],
  { stdio: ['pipe', 'pipe', 'inherit'] }
)
const pending = new Map()
let sequence = 0
createInterface({ input: child.stdout }).on('line', (line) => {
  const value = JSON.parse(line)
  const request = pending.get(value.id)
  if (request) {
    clearTimeout(request.timeout)
    pending.delete(value.id)
    request.resolve(value)
  }
})
function call(request) {
  return new Promise((resolve, reject) => {
    const id = ++sequence
    const timeout = setTimeout(() => {
      pending.delete(id)
      reject(new Error('Native verification request timed out'))
    }, 30000)
    pending.set(id, { resolve, reject, timeout })
    child.stdin.write(JSON.stringify({ id, request }) + '\n')
  })
}
const origin = 'http://127.0.0.1:15422'
const server = await createServer({
  configFile: false,
  root: resolve('app'),
  cacheDir: resolve('.tooling/vite-smoke'),
  define: { 'import.meta.env.VITE_ZENITH_TEST_BRIDGE': JSON.stringify('1') },
  plugins: [
    react(),
    {
      name: 'zenith-verification-only',
      configureServer(server) {
        server.middlewares.use('/__zenith_test_ipc', async (req, res) => {
          if (
            req.method !== 'POST' ||
            req.headers.origin !== origin ||
            !req.headers['content-type']?.startsWith('application/json')
          ) {
            res.statusCode = 403
            res.end()
            return
          }
          let body = ''
          req.on('data', (chunk) => {
            body += chunk
            if (body.length > 16384) req.destroy()
          })
          req.on('end', async () => {
            try {
              const result = await call(JSON.parse(body))
              res.setHeader('Content-Type', 'application/json')
              res.end(JSON.stringify(result))
            } catch (error) {
              res.statusCode = 400
              res.end(JSON.stringify({ error: error.message }))
            }
          })
        })
      },
    },
  ],
  server: { host: '127.0.0.1', port: 15422, strictPort: true },
})
await server.listen()
console.log(
  `UI verification: ${origin}\nIsolated database: ${db}\nTransport: React → same-origin development bridge → real Rust commands → SQLite`
)
async function stop() {
  await server.close()
  child.stdin.end()
  process.exit()
}
process.on('SIGINT', stop)
process.on('SIGTERM', stop)
