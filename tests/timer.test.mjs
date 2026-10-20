import test from 'node:test'
import assert from 'node:assert/strict'
import { timeLeft, clockText, duration } from '../app/format.js'

test('elapsed time comes from persisted timestamps, including suspension', () => {
  const s = { planned_seconds: 1500, elapsed_ms: 120000, running_since: 1000000, status: 'running' }
  assert.equal(timeLeft(s, 1060000), 1320000)
  assert.equal(timeLeft(s, 4000000), 0)
  assert.equal(timeLeft({ ...s, status: 'paused', running_since: null }, 4000000), 1380000)
  assert.equal(timeLeft(s, 999000), 1380000)
})
test('timer display rounds up and never shows negative time', () => {
  assert.equal(clockText(1500000), '25:00')
  assert.equal(clockText(1), '00:01')
  assert.equal(clockText(0), '00:00')
  assert.equal(clockText(5400000), '90:00')
  assert.equal(duration(3660), '1h 1m')
})
