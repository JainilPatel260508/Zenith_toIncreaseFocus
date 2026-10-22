import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  root: 'app',
  plugins: [react()],
  server: { host: '127.0.0.1', port: 15420, strictPort: true },
  build: { outDir: '../dist', emptyOutDir: true, sourcemap: false },
})
