import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { spawnSync } from 'node:child_process'

const env = { ...process.env }
if (existsSync('.tooling/cargo/bin/cargo')) {
  env.CARGO_HOME = resolve('.tooling/cargo')
  env.RUSTUP_HOME = resolve('.tooling/rustup')
  env.PATH = `${env.CARGO_HOME}/bin:${env.PATH}`
}
const result = spawnSync(resolve('node_modules/.bin/tauri'), process.argv.slice(2), { env, stdio: 'inherit' })
if (result.error) console.error(result.error.message)
process.exit(result.status ?? 1)
