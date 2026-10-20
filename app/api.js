import { invoke, isTauri } from '@tauri-apps/api/core'

export const verification = import.meta.env.DEV && import.meta.env.VITE_ZENITH_TEST_BRIDGE === '1'
export const native = isTauri() || verification
export async function api(action, fields = {}) {
  if (!native)
    throw new Error(
      'Open the Zenith desktop app to use your local workspace. Run npm run desktop from the project folder.'
    )
  try {
    if (verification) {
      const response = await fetch('/__zenith_test_ipc', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action, ...fields }),
      })
      const result = await response.json()
      if (result.error) throw new Error(result.error)
      return result.result
    }
    return await invoke('request', { request: { action, ...fields } })
  } catch (error) {
    throw new Error(
      typeof error === 'string' ? error : error.message || 'Something went wrong. Please try again.'
    )
  }
}
