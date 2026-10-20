import Icon from './Icons'
import { timestamp } from './format'

export default function Achievements({ data }) {
  const unlocked = data.achievements.filter((a) => a.unlocked_at).length
  return (
    <>
      <section className="card reward-banner">
        <div className="reward-emblem">
          <Icon name="award" size={40} />
        </div>
        <div>
          <div className="eyebrow">EFFORT WORTH CELEBRATING</div>
          <h2>Every focused moment counts.</h2>
          <p className="muted">Keep showing up. Your milestones will follow.</p>
        </div>
        <div className="point-balance">
          <strong>{data.points}</strong>
          <span>FOCUS POINTS</span>
        </div>
      </section>
      <div className="section-toolbar">
        <h2>Your milestones</h2>
        <span className="status green">
          {unlocked} of {data.achievements.length} unlocked
        </span>
      </div>
      <div className="achievements-grid">
        {data.achievements.map((a, i) => (
          <section
            key={a.id}
            className={`card achievement-card ${a.unlocked_at ? 'unlocked' : ''}`}
          >
            <div className="card-heading">
              <span className="achievement-icon">
                <Icon name={['leaf', 'bolt', 'timer', 'flame', 'award'][i]} size={28} />
              </span>
              {a.unlocked_at ? (
                <span className="status green">
                  <Icon name="check" size={11} /> Unlocked
                </span>
              ) : (
                <Icon name="lock" size={15} />
              )}
            </div>
            <h2>{a.name}</h2>
            <p className="muted">{a.description}</p>
            <div
              className="progress-track"
              role="progressbar"
              aria-label={a.name}
              aria-valuemin={0}
              aria-valuemax={a.target}
              aria-valuenow={a.progress}
            >
              <span style={{ width: `${(a.progress / a.target) * 100}%` }} />
            </div>
            <div className="achievement-meta">
              {a.unlocked_at ? (
                <span>
                  Earned{' '}
                  {new Date(a.unlocked_at).toLocaleDateString([], {
                    month: 'short',
                    day: 'numeric',
                    year: 'numeric',
                  })}
                </span>
              ) : (
                <>
                  <span>
                    {a.progress} / {a.target} {a.metric === 'streak' ? 'days' : a.metric}
                  </span>
                  <span>{Math.round((a.progress / a.target) * 100)}%</span>
                </>
              )}
            </div>
          </section>
        ))}
      </div>
      <section className="card history">
        <div className="card-heading">
          <div>
            <h2>Your points ledger</h2>
            <p className="muted small">
              Each eligible session earns its configured reward exactly once.
            </p>
          </div>
          <Icon name="bolt" />
        </div>
        {data.transactions.length ? (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Reward</th>
                  <th>Date</th>
                  <th>Session</th>
                  <th>Points</th>
                </tr>
              </thead>
              <tbody>
                {data.transactions.map((t) => (
                  <tr key={`${t.session_id}-${t.created_at}`}>
                    <td>
                      <strong>{t.reason}</strong>
                    </td>
                    <td>{timestamp(t.created_at)}</td>
                    <td>#{t.session_id}</td>
                    <td className="accent">+{t.amount}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="empty">
            <Icon name="bolt" size={28} />
            <h3>Your effort has value.</h3>
            <p>Finish your first focus session to earn {data.user.session_points} points.</p>
          </div>
        )}
        <div className="notice">
          Focus sessions of at least 1 minute earn the points chosen in settings when they start
          (currently {data.user.session_points}). Breaks and cancelled sessions earn zero.
          Achievements are badges; they do not add bonus points. Points have no monetary value.
        </div>
      </section>
    </>
  )
}
