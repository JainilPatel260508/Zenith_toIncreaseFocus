import { useState } from 'react'
import { api, native, verification } from './api'

export default function Auth({ hasAccounts, onLogin }) {
  const [register, setRegister] = useState(!hasAccounts)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  async function submit(e) {
    e.preventDefault()
    setBusy(true)
    setError('')
    const f = new FormData(e.currentTarget)
    try {
      const data = await api(register ? 'register' : 'login', {
        email: f.get('email'),
        password: f.get('password'),
        ...(register ? { display_name: f.get('display_name') } : {}),
      })
      onLogin(data)
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }
  return (
    <div className="auth-shell">
      <section className="auth-story">
        <div className="brand">
          <span className="brand-mark">Z</span>zenith<span className="brand-dot">.</span>
        </div>
        <div>
          <div className="eyebrow">LESS DISTRACTION. MORE INTENTION.</div>
          <h1>
            A little more focus.
            <br />A lot more possibility.
          </h1>
          <p>
            A quiet place for your most meaningful work.
            <br />
            Build a habit, find your rhythm, and watch it add up.
          </p>
          <div className="auth-art" aria-hidden="true">
            <div />
            <div />
            <div />
            <span>ONE SESSION AT A TIME</span>
          </div>
        </div>
        <small>Private by design. Your workspace stays on this Mac.</small>
      </section>
      <section className="auth-form">
        <div className="eyebrow">YOUR PERSONAL WORKSPACE</div>
        <h1>{register ? 'Start your next chapter.' : 'Welcome back.'}</h1>
        <p className="muted">
          {register
            ? 'Create a local account to make space for focus.'
            : 'Your focus, habits, and progress are right here.'}
        </p>
        {!native && (
          <div className="notice">
            This is the interface preview. Launch the desktop app with <code>npm run desktop</code>{' '}
            to use SQLite and local accounts.
          </div>
        )}
        {verification && (
          <div className="notice">
            Verification workspace · Connected to the real Rust backend with an isolated test
            database.
          </div>
        )}
        <form onSubmit={submit}>
          {register && (
            <label>
              Display name
              <input
                name="display_name"
                autoComplete="name"
                required
                maxLength={60}
                placeholder="What should we call you?"
              />
            </label>
          )}
          <label>
            Email address
            <input
              name="email"
              type="email"
              autoComplete="username"
              required
              maxLength={254}
              placeholder="you@example.com"
            />
          </label>
          <label>
            Password
            <input
              name="password"
              type="password"
              autoComplete={register ? 'new-password' : 'current-password'}
              required
              minLength={register ? 10 : 1}
              maxLength={128}
              placeholder={register ? 'At least 10 characters' : 'Your password'}
            />
          </label>
          {error && (
            <div className="error" role="alert">
              {error}
            </div>
          )}
          <button disabled={busy || !native} className="primary wide">
            {busy ? 'One moment…' : register ? 'Create your workspace →' : 'Sign in →'}
          </button>
        </form>
        <p className="muted">
          {register ? 'Already have a local account?' : 'New to this Mac?'}{' '}
          <button
            className="text-button"
            onClick={() => {
              setRegister(!register)
              setError('')
            }}
          >
            {register ? 'Sign in' : 'Create an account'}
          </button>
        </p>
        <div className="auth-note">
          LOCAL ACCOUNT · NO CLOUD REQUIRED
          <p>
            Your password is securely hashed. No email is sent, and your account is available only
            on this Mac.
          </p>
        </div>
      </section>
    </div>
  )
}
