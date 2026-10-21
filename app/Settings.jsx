import { useState } from 'react'

export default function Settings({ user, act }) {
  const [saved,setSaved]=useState(false)
  async function save(e) {
    e.preventDefault(); setSaved(false)
    const f=new FormData(e.currentTarget)
    const ok=await act('save_settings',{display_name:f.get('display_name'),email:f.get('email'),focus_minutes:Number(f.get('focus_minutes')),short_break:Number(f.get('short_break')),long_break:Number(f.get('long_break')),theme:f.get('theme'),notifications:f.has('notifications'),session_points:Number(f.get('session_points'))})
    if(ok)setSaved(true)
  }
  return <form className="settings-grid" onSubmit={save}><section className="card"><h2>Your profile</h2><p className="muted">Make this workspace feel like you.</p><label>Display name<input name="display_name" defaultValue={user.display_name} required maxLength={60}/></label><label>Email address<input name="email" type="email" defaultValue={user.email} required maxLength={254}/></label><label>Appearance<select name="theme" defaultValue={user.theme}><option value="dark">Dark · Quiet hours</option><option value="light">Light · Fresh start</option></select></label><label className="check"><input name="notifications" type="checkbox" defaultChecked={user.notifications}/>Play a gentle completion sound</label><p className="muted small">In-app completion messages always appear. macOS push notifications are not used.</p></section><section className="card"><h2>Your focus rhythm</h2><p className="muted">A starting point for each new session.</p><div className="form-row"><label>Focus (minutes)<input name="focus_minutes" type="number" min="1" max="180" required defaultValue={user.focus_minutes}/></label><label>Short break<input name="short_break" type="number" min="1" max="60" required defaultValue={user.short_break}/></label><label>Long break<input name="long_break" type="number" min="1" max="120" required defaultValue={user.long_break}/></label></div><label>Points per completed focus session<input name="session_points" type="number" min="1" max="100" required defaultValue={user.session_points}/></label><div className="notice">Your reward is fixed when a session starts. Breaks and cancelled sessions earn no points.</div><button className="primary" type="submit">Save preferences</button>{saved&&<span className="success" role="status"> Preferences saved</span>}</section></form>
}
