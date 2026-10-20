import { useEffect, useState } from 'react'
import { clockText, timeLeft, kindName, timestamp, duration } from './format'
import Icon from './Icons'

function useNow() {
  const [now, setNow] = useState(Date.now)
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 250)
    return () => clearInterval(id)
  }, [])
  return now
}
export default function Timer({ data, act, compact = false, onNavigate }) {
  const session = data.active_session
  const [mode, setMode] = useState('focus')
  const [custom, setCustom] = useState(data.user.focus_minutes)
  const [confirm, setConfirm] = useState(false)
  const now = useNow()
  const currentMode = session?.kind || mode
  const planned =
    session?.planned_seconds ||
    60 *
      (mode === 'focus'
        ? custom
        : mode === 'short_break'
          ? data.user.short_break
          : data.user.long_break)
  const remaining = session ? timeLeft(session, now) : planned * 1000
  const progress = session ? Math.min(1, 1 - remaining / (planned * 1000)) : 0
  const start = () => act('start_session', { minutes: planned / 60, kind: mode, room_id: null })
  const operation = (op) => act('session_action', { session_id: session.id, operation: op })
  return (
    <section className={`card timer-card ${compact ? 'compact' : ''}`}>
      <div className="card-heading">
        <div>
          <span className="eyebrow">{compact ? 'MAKE THIS MOMENT COUNT' : 'YOUR FOCUS SPACE'}</span>
          <h2>{session?.room_name || 'One thing at a time.'}</h2>
        </div>
        <span className={`status ${session ? 'green' : ''}`}>
          {session
            ? session.status === 'running'
              ? 'In progress'
              : 'Paused'
            : 'Ready when you are'}
        </span>
      </div>
      <div className="segmented" aria-label="Session type">
        {[
          ['focus', 'Focus'],
          ['short_break', 'Short break'],
          ['long_break', 'Long break'],
        ].map(([id, name]) => (
          <button
            key={id}
            disabled={!!session}
            className={currentMode === id ? 'selected' : ''}
            aria-pressed={currentMode === id}
            onClick={() => setMode(id)}
          >
            {name}
          </button>
        ))}
      </div>
      <div
        className="timer-ring"
        aria-label={`${kindName(currentMode)}: ${clockText(remaining)} remaining`}
      >
        <svg viewBox="0 0 250 250" aria-hidden="true">
          <circle className="ring-track" cx="125" cy="125" r="110" />
          <circle
            className="ring-progress"
            cx="125"
            cy="125"
            r="110"
            strokeDasharray={691.15}
            strokeDashoffset={691.15 * (1 - progress)}
          />
          <circle className="ring-inner" cx="125" cy="125" r="97" />
        </svg>
        <div className="timer-face">
          <span className="eyebrow">{currentMode === 'focus' ? 'DEEP WORK' : 'TAKE A BREATH'}</span>
          <strong>{clockText(remaining)}</strong>
          <span>
            {session?.status === 'paused'
              ? 'Your progress is safe'
              : currentMode === 'focus'
                ? 'Let the rest wait.'
                : 'Rest is part of the process.'}
          </span>
        </div>
      </div>
      {!session && mode === 'focus' && (
        <div className="timer-presets">
          {[25, 50, 90].map((m) => (
            <button
              key={m}
              aria-pressed={custom === m}
              className={custom === m ? 'chosen' : ''}
              onClick={() => setCustom(m)}
            >
              {m} min
            </button>
          ))}
          {!compact && (
            <label className="custom-duration">
              <input
                aria-label="Custom focus duration in minutes"
                type="number"
                min="1"
                max="180"
                value={custom}
                onChange={(e) => setCustom(Number(e.target.value))}
              />{' '}
              min
            </label>
          )}
        </div>
      )}
      <div className="timer-actions">
        {!session ? (
          <button
            className="primary"
            onClick={start}
            disabled={!Number.isInteger(custom) || custom < 1 || custom > 180}
          >
            <Icon name="play" />
            {currentMode === 'focus' ? 'Start focusing' : 'Start break'}
          </button>
        ) : (
          <>
            <button
              className="primary"
              onClick={() => operation(session.status === 'running' ? 'pause' : 'resume')}
            >
              <Icon name={session.status === 'running' ? 'pause' : 'play'} />
              {session.status === 'running' ? 'Pause session' : 'Resume session'}
            </button>
            <button
              className="secondary icon-button"
              aria-label="Reset session"
              onClick={() => setConfirm(true)}
            >
              <Icon name="reset" />
            </button>
          </>
        )}
      </div>
      {confirm && session && (
        <div className="notice" role="alert">
          Reset this session? Your time will be recorded as cancelled, with no points.
          <div className="button-row">
            <button
              className="danger"
              onClick={async () => {
                if (await operation('cancel')) setConfirm(false)
              }}
            >
              Reset session
            </button>
            <button className="secondary" onClick={() => setConfirm(false)}>
              Keep focusing
            </button>
          </div>
        </div>
      )}
      <div className="timer-foot">
        <Icon name="shield" size={14} />
        {data.user.focus_mode
          ? 'Zenith resource protection is on'
          : 'Zenith resource protection is off'}
        {compact && (
          <button className="text-button" onClick={() => onNavigate('Focus Timer')}>
            Customize <Icon name="arrow" size={13} />
          </button>
        )}
      </div>
      {session && (
        <div className="muted small center">
          Saved automatically · Running timers continue while Zenith is closed.
        </div>
      )}
    </section>
  )
}
export function SessionHistory({ sessions, limit = 20 }) {
  const [filter, setFilter] = useState('all')
  const filtered = sessions.filter((s) => filter === 'all' || s.status === filter).slice(0, limit)
  return (
    <section className="card history">
      <div className="card-heading">
        <div>
          <h2>Session history</h2>
          <p className="muted small">Your most recent sessions, saved on this Mac.</p>
        </div>
        <select
          aria-label="Filter sessions"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        >
          <option value="all">All sessions</option>
          <option value="completed">Completed</option>
          <option value="cancelled">Cancelled</option>
        </select>
      </div>
      {filtered.length ? (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Session</th>
                <th>Started</th>
                <th>Duration</th>
                <th>Status</th>
                <th>Points</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((s) => (
                <tr key={s.id}>
                  <td>
                    <strong>{s.room_name || kindName(s.kind)}</strong>
                    <span className="table-sub">
                      {s.room_name
                        ? 'Room session'
                        : s.kind === 'focus'
                          ? 'Personal focus'
                          : 'Recovery time'}
                    </span>
                  </td>
                  <td>{timestamp(s.started_at)}</td>
                  <td>
                    {duration(s.actual_seconds)}{' '}
                    <span className="muted">/ {duration(s.planned_seconds)}</span>
                  </td>
                  <td>
                    <span className={`status ${s.status === 'completed' ? 'green' : ''}`}>
                      {s.status}
                    </span>
                  </td>
                  <td className={s.points_awarded ? 'accent' : ''}>
                    {s.points_awarded ? `+${s.points_awarded}` : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="empty">
          <Icon name="timer" size={28} />
          <h3>Your next session starts a story.</h3>
          <p>Completed and cancelled sessions will appear here.</p>
        </div>
      )}
    </section>
  )
}
