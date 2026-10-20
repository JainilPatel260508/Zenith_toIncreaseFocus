let context
export function prepareSound() {
  try {
    context ||= new (window.AudioContext || window.webkitAudioContext)()
    if (context.state === 'suspended') context.resume().catch(() => {})
  } catch {
    /* Audio is optional; completion is always shown visually. */
  }
}
export function playCompletion() {
  try {
    if (!context || context.state !== 'running') return
    const oscillator = context.createOscillator()
    const gain = context.createGain()
    oscillator.connect(gain)
    gain.connect(context.destination)
    oscillator.type = 'sine'
    oscillator.frequency.value = 660
    gain.gain.setValueAtTime(0.08, context.currentTime)
    gain.gain.exponentialRampToValueAtTime(0.001, context.currentTime + 0.7)
    oscillator.start()
    oscillator.stop(context.currentTime + 0.7)
  } catch {
    /* Visual notification remains available. */
  }
}
