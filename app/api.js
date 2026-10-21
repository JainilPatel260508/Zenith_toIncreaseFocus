import { invoke, isTauri } from '@tauri-apps/api/core'

export const native = isTauri()
export async function api(action, fields = {}) {
  if (!native) throw new Error('Open the Zenith desktop app to use your local workspace. Run npm run desktop from the project folder.')
  try { return await invoke('request', { request: { action, ...fields } }) }
  catch (error) { throw new Error(typeof error === 'string' ? error : error.message || 'Something went wrong. Please try again.') }
}
