import { useCallback, useEffect, useRef, useState } from 'react'
import './App.css'

const WHITE_NOTES = [
  { name: 'C', semitone: 0, shortcut: 'A' },
  { name: 'D', semitone: 2, shortcut: 'S' },
  { name: 'E', semitone: 4, shortcut: 'D' },
  { name: 'F', semitone: 5, shortcut: 'F' },
  { name: 'G', semitone: 7, shortcut: 'G' },
  { name: 'A', semitone: 9, shortcut: 'H' },
  { name: 'B', semitone: 11, shortcut: 'J' },
  { name: 'C', semitone: 12, shortcut: 'K' },
  { name: 'D', semitone: 14, shortcut: 'L' },
  { name: 'E', semitone: 16, shortcut: ';' },
  { name: 'F', semitone: 17, shortcut: "'" },
  { name: 'G', semitone: 19, shortcut: '' },
  { name: 'A', semitone: 21, shortcut: '' },
  { name: 'B', semitone: 23, shortcut: '' },
]

const BLACK_NOTES = [
  { name: 'C♯', semitone: 1, after: 0, shortcut: 'W' },
  { name: 'D♯', semitone: 3, after: 1, shortcut: 'E' },
  { name: 'F♯', semitone: 6, after: 3, shortcut: 'T' },
  { name: 'G♯', semitone: 8, after: 4, shortcut: 'Y' },
  { name: 'A♯', semitone: 10, after: 5, shortcut: 'U' },
  { name: 'C♯', semitone: 13, after: 7, shortcut: 'O' },
  { name: 'D♯', semitone: 15, after: 8, shortcut: 'P' },
  { name: 'F♯', semitone: 18, after: 10, shortcut: '[' },
  { name: 'G♯', semitone: 20, after: 11, shortcut: ']' },
  { name: 'A♯', semitone: 22, after: 12, shortcut: '\\' },
]

const INSTRUMENTS = [
  { id: 'organ', label: 'Classic organ', icon: '▥' },
  { id: 'warm', label: 'Warm keys', icon: '◒' },
  { id: 'synth', label: 'Soft synth', icon: '〰' },
]

const KEYBOARD_NOTES = [
  ...WHITE_NOTES.map((note) => ({ ...note, color: 'white' })),
  ...BLACK_NOTES.map((note) => ({ ...note, color: 'black' })),
]
const KEY_MAP = new Map(
  KEYBOARD_NOTES.filter((note) => note.shortcut).map((note) => [
    note.shortcut.toLowerCase(),
    note.semitone,
  ]),
)

function App() {
  const [instrument, setInstrument] = useState('organ')
  const [volume, setVolume] = useState(68)
  const [octave, setOctave] = useState(4)
  const [activeNotes, setActiveNotes] = useState(() => new Set())
  const [lastNote, setLastNote] = useState('Ready to play')
  const audioRef = useRef(null)
  const masterRef = useRef(null)
  const voicesRef = useRef(new Map())
  const octaveRef = useRef(octave)

  useEffect(() => {
    octaveRef.current = octave
  }, [octave])

  useEffect(() => {
    if (masterRef.current) {
      masterRef.current.gain.setTargetAtTime(
        volume / 100,
        audioRef.current.currentTime,
        0.025,
      )
    }
  }, [volume])

  const getAudio = useCallback(() => {
    if (!audioRef.current) {
      const AudioContextClass =
        window.AudioContext || window.webkitAudioContext
      if (!AudioContextClass) {
        throw new Error('Web Audio is not supported in this browser.')
      }
      const context = new AudioContextClass()
      const master = context.createGain()
      master.gain.value = volume / 100
      master.connect(context.destination)
      audioRef.current = context
      masterRef.current = master
    }
    if (audioRef.current.state === 'suspended') {
      audioRef.current.resume()
    }
    return audioRef.current
  }, [volume])

  const startNote = useCallback(
    (semitone) => {
      const midi = 12 * (octaveRef.current + 1) + semitone
      if (voicesRef.current.has(midi)) return

      try {
        const context = getAudio()
        const voice = context.createGain()
        const now = context.currentTime
        voice.gain.setValueAtTime(0.0001, now)
        voice.gain.exponentialRampToValueAtTime(
          instrument === 'organ' ? 0.3 : 0.22,
          now + 0.025,
        )
        voice.connect(masterRef.current)

        const oscillators =
          instrument === 'organ'
            ? [
                { type: 'sine', ratio: 1, gain: 0.58 },
                { type: 'sine', ratio: 2, gain: 0.2 },
                { type: 'sine', ratio: 3, gain: 0.1 },
              ]
            : instrument === 'warm'
              ? [
                  { type: 'triangle', ratio: 1, gain: 0.8 },
                  { type: 'sine', ratio: 2, gain: 0.12 },
                ]
              : [
                  { type: 'sawtooth', ratio: 1, gain: 0.35 },
                  { type: 'sine', ratio: 2, gain: 0.15 },
                ]

        const nodes = oscillators.map(({ type, ratio, gain }) => {
          const oscillator = context.createOscillator()
          const partialGain = context.createGain()
          oscillator.type = type
          oscillator.frequency.value = 440 * 2 ** ((midi - 69) / 12) * ratio
          partialGain.gain.value = gain
          oscillator.connect(partialGain)
          partialGain.connect(voice)
          oscillator.start(now)
          return oscillator
        })

        voicesRef.current.set(midi, { gain: voice, oscillators: nodes })
        setActiveNotes((current) => new Set(current).add(midi))
        const noteName = noteNameFor(midi)
        setLastNote(`${noteName}  ·  ${instrumentLabel(instrument)}`)
      } catch (error) {
        setLastNote(error.message)
      }
    },
    [getAudio, instrument],
  )

  const stopNote = useCallback((semitone) => {
    const midi = 12 * (octaveRef.current + 1) + semitone
    const voice = voicesRef.current.get(midi)
    if (!voice) return

    const context = audioRef.current
    const now = context.currentTime
    voice.gain.gain.cancelScheduledValues(now)
    voice.gain.gain.setTargetAtTime(0.0001, now, 0.08)
    voice.oscillators.forEach((oscillator) => oscillator.stop(now + 0.4))
    voicesRef.current.delete(midi)
    setActiveNotes((current) => {
      const next = new Set(current)
      next.delete(midi)
      return next
    })
  }, [])

  useEffect(() => {
    const handleKeyDown = (event) => {
      if (
        event.repeat ||
        event.metaKey ||
        event.ctrlKey ||
        event.altKey ||
        event.target instanceof HTMLInputElement
      ) {
        return
      }
      const semitone = KEY_MAP.get(event.key.toLowerCase())
      if (semitone === undefined) return
      event.preventDefault()
      startNote(semitone)
    }
    const handleKeyUp = (event) => {
      const semitone = KEY_MAP.get(event.key.toLowerCase())
      if (semitone !== undefined) stopNote(semitone)
    }
    const releaseAll = () => {
      voicesRef.current.forEach((_, midi) => {
        stopNote(midi - 12 * (octaveRef.current + 1))
      })
    }

    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)
    window.addEventListener('blur', releaseAll)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
      window.removeEventListener('keyup', handleKeyUp)
      window.removeEventListener('blur', releaseAll)
    }
  }, [startNote, stopNote])

  const changeOctave = (direction) => {
    voicesRef.current.forEach((_, midi) => {
      stopNote(midi - 12 * (octaveRef.current + 1))
    })
    setOctave((current) => Math.min(6, Math.max(2, current + direction)))
  }

  const renderKey = (note, isBlack = false) => {
    const midi = 12 * (octave + 1) + note.semitone
    const isActive = activeNotes.has(midi)
    return (
      <button
        key={note.semitone}
        type="button"
        className={`key ${isBlack ? 'key-black' : 'key-white'} ${isActive ? 'key-active' : ''}`}
        style={
          isBlack
            ? { left: `${((note.after + 1) / WHITE_NOTES.length) * 100}%` }
            : undefined
        }
        aria-label={`${note.name}${Math.floor(octave + note.semitone / 12)}${note.shortcut ? `, keyboard ${note.shortcut}` : ''}`}
        onPointerDown={(event) => {
          event.preventDefault()
          event.currentTarget.setPointerCapture(event.pointerId)
          startNote(note.semitone)
        }}
        onPointerUp={() => stopNote(note.semitone)}
        onPointerCancel={() => stopNote(note.semitone)}
      >
        {!isBlack && (
          <span className="key-label">
            {note.semitone === 0 || note.semitone === 12 ? note.name : ''}
          </span>
        )}
        <span className={`key-shortcut ${isBlack ? 'shortcut-black' : ''}`}>
          {note.shortcut}
        </span>
      </button>
    )
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <a className="brand" href="/" aria-label="Electone home">
          <span className="brand-mark" aria-hidden="true">
            <span />
            <span />
            <span />
          </span>
          <span className="brand-name">electone</span>
        </a>
        <div className="topbar-right">
          <span className="live-indicator"><span /> LIVE INSTRUMENT</span>
          <button
            className="sound-status"
            type="button"
            onClick={() => {
              try {
                getAudio()
                setLastNote('Sound is ready')
              } catch (error) {
                setLastNote(error.message)
              }
            }}
          >
            <span className="sound-icon" aria-hidden="true">◖</span>
            Sound on
          </button>
        </div>
      </header>

      <section className="intro">
        <div className="eyebrow"><span /> YOUR LITTLE MUSIC ROOM</div>
        <h1>Make a little <em>music.</em></h1>
        <p>A tiny organ for wherever the moment takes you.</p>
      </section>

      <section className="instrument-panel" aria-label="Electronic organ">
        <div className="panel-top">
          <div className="instrument-title">
            <span className="panel-icon">♫</span>
            <div>
              <h2>The Electone</h2>
              <span className="panel-subtitle">A little room to play</span>
            </div>
          </div>
          <div className="note-display" aria-live="polite">
            <span className="note-display-dot" />
            {lastNote}
          </div>
        </div>

        <div className="controls">
          <div className="tone-control">
            <span className="control-label">SOUND</span>
            <div className="tone-options" role="group" aria-label="Instrument sound">
              {INSTRUMENTS.map((sound) => (
                <button
                  key={sound.id}
                  type="button"
                  className={`tone-option ${instrument === sound.id ? 'selected' : ''}`}
                  onClick={() => setInstrument(sound.id)}
                  aria-pressed={instrument === sound.id}
                >
                  <span className="tone-icon" aria-hidden="true">{sound.icon}</span>
                  {sound.label}
                </button>
              ))}
            </div>
          </div>

          <div className="volume-control">
            <label htmlFor="volume" className="control-label">VOLUME</label>
            <div className="volume-row">
              <span className="volume-icon" aria-hidden="true">⌁</span>
              <input
                id="volume"
                type="range"
                min="0"
                max="100"
                value={volume}
                style={{ '--volume-fill': `${volume}%` }}
                onChange={(event) => setVolume(Number(event.target.value))}
                aria-label="Volume"
              />
              <span className="volume-value">{volume}%</span>
            </div>
          </div>
        </div>

        <div className="keyboard-heading">
          <div>
            <span className="control-label">THE KEYS</span>
            <span className="keyboard-hint">Tap a key or use your keyboard</span>
          </div>
          <div className="octave-control" aria-label="Octave control">
            <span>OCTAVE</span>
            <button
              type="button"
              onClick={() => changeOctave(-1)}
              disabled={octave <= 2}
              aria-label="Lower octave"
            >−</button>
            <b>{octave}</b>
            <button
              type="button"
              onClick={() => changeOctave(1)}
              disabled={octave >= 6}
              aria-label="Raise octave"
            >+</button>
          </div>
        </div>

        <div className="keyboard" aria-label={`Keyboard, octave ${octave}`}>
          <div className="white-keys">
            {WHITE_NOTES.map((note) => renderKey(note))}
          </div>
          <div className="black-keys">
            {BLACK_NOTES.map((note) => renderKey(note, true))}
          </div>
        </div>
        <div className="keyboard-footer">
          <span><i className="footer-dot" /> READY WHEN YOU ARE</span>
          <span className="key-legend">A–; WHITE KEYS <span>·</span> W–\ BLACK KEYS</span>
        </div>
      </section>

      <footer className="page-footer">
        <span>Take a breath. Find a note. See what happens.</span>
        <span className="footer-flower" aria-hidden="true">✳</span>
      </footer>
    </main>
  )
}

function instrumentLabel(id) {
  return INSTRUMENTS.find((sound) => sound.id === id)?.label ?? 'Classic organ'
}

function noteNameFor(midi) {
  const names = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B']
  return `${names[midi % 12]}${Math.floor(midi / 12) - 1}`
}

export default App
