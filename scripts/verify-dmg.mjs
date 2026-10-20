import { statSync, existsSync, openSync, readSync, closeSync } from 'node:fs'
import { resolve } from 'node:path'
const file = resolve(
  process.argv[2] ||
    'src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Zenith_1.0.0_aarch64.dmg'
)
if (!file.endsWith('.dmg') || !existsSync(file)) {
  console.error(`FAIL: DMG not found: ${file}`)
  process.exit(1)
}
const size = statSync(file).size
const fd = openSync(file, 'r')
const signature = Buffer.alloc(4)
if (size >= 512) readSync(fd, signature, 0, 4, size - 512)
closeSync(fd)
console.log(
  `DMG: ${file}\nSize: ${size} bytes\nDecimal MB: ${(size / 1_000_000).toFixed(6)}\nLimit: strictly less than 50000000 bytes`
)
if (signature.toString() !== 'koly') {
  console.error('FAIL: Missing Apple UDIF disk-image trailer')
  process.exit(1)
}
if (size >= 50_000_000) {
  console.error('FAIL: Installer exceeds the strict size limit')
  process.exit(1)
}
console.log(
  'PASS: Valid UDIF signature and installer below 50 MB. Run hdiutil verify and mount it to verify integrity and launchability.'
)
