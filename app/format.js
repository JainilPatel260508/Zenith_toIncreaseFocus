export const minutes = seconds => Math.round(seconds / 60)
export const duration = seconds => { const m=Math.floor(seconds/60); return m>=60 ? `${Math.floor(m/60)}h ${m%60}m` : `${m}m` }
export const timestamp = ms => new Date(ms).toLocaleString([], {month:'short',day:'numeric',hour:'2-digit',minute:'2-digit'})
export const timeLeft = (session, now) => session ? Math.max(0, session.planned_seconds * 1000 - session.elapsed_ms - (session.status === 'running' ? Math.max(0, now - session.running_since) : 0)) : 0
export const clockText = ms => { const seconds=Math.ceil(ms/1000); return `${Math.floor(seconds/60).toString().padStart(2,'0')}:${(seconds%60).toString().padStart(2,'0')}` }
export const kindName = kind => ({focus:'Focus session',short_break:'Short break',long_break:'Long break'}[kind]||kind)
