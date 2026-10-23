import { useCallback, useEffect, useState } from 'react'
import { api, native } from './api'
import Auth from './Auth'
import Settings from './Settings'
import Timer, { SessionHistory } from './Timer'
import Icon from './Icons'
import Rooms from './Rooms'

const iconNames=['dashboard','timer','rooms','chart','shield','award','settings']
const pages=['Dashboard','Focus Timer','Focus Rooms','Activity & Analytics','Website Blocker','Achievements','Profile & Settings']
export default function App() {
  const [page,setPage]=useState('Dashboard')
  const [data,setData]=useState(null)
  const [boot,setBoot]=useState(native ? null : {has_accounts:false})
  const [error,setError]=useState('')
  const [busy,setBusy]=useState(false)
  const userId=data?.user.id; const activeId=data?.active_session?.id
  useEffect(()=>{if(!userId)return;let live=true; const refresh=()=>api('snapshot').then(next=>{if(live)setData(current=>current&&next.server_now>=current.server_now?next:current)}).catch(e=>{if(live)setError(e.message)});const id=setInterval(refresh,activeId?1000:15000);window.addEventListener('focus',refresh);return()=>{live=false;clearInterval(id);window.removeEventListener('focus',refresh)}},[userId,activeId])
  useEffect(()=>{ if(!native)return; api('bootstrap').then(setBoot).catch(e=>setError(e.message)) },[])
  useEffect(()=>{document.documentElement.dataset.theme=data?.user.theme||'dark'},[data?.user.theme])
  const act=useCallback(async(action,fields={})=>{
    setBusy(true);setError('')
    try{const next=await api(action,fields);setData(next);return next}catch(e){setError(e.message);return null}finally{setBusy(false)}
  },[])
  async function logout(){try{await api('logout');setData(null);setBoot({has_accounts:true});setPage('Dashboard')}catch(e){setError(e.message)}}
  if(!boot)return <div className="loading"><span className="brand-mark">Z</span><p>{error||'Opening your workspace…'}</p>{error&&<button onClick={()=>location.reload()}>Try again</button>}</div>
  if(!data)return <Auth hasAccounts={boot.has_accounts} onLogin={setData}/>
  return <div className="shell"><aside><div className="brand"><span className="brand-mark">Z</span>zenith<span className="brand-dot">.</span></div><div className="nav-label">YOUR WORKSPACE</div><nav aria-label="Main navigation">{pages.map((item,i)=><button key={item} className={page===item?'active':''} onClick={()=>setPage(item)} aria-current={page===item?'page':undefined}><Icon name={iconNames[i]}/>{item}</button>)}</nav><footer>Made for meaningful work.<br/><small>LOCAL WORKSPACE · v1.0</small><button className="profile-button" onClick={()=>setPage('Profile & Settings')}><span className="avatar">{data.user.display_name.slice(0,1)}</span><span>{data.user.display_name}<small>Personal account</small></span></button><button className="text-button" onClick={logout}>Sign out</button></footer></aside><main><header><span>Workspace <span className="breadcrumb">/ {page}</span></span><span className="badge">● Saved on this Mac</span></header><div className="page"><div className="eyebrow">A LITTLE MORE FOCUS. EVERY DAY.</div><h1>{page==='Dashboard'?`Make room for your best work, ${data.user.display_name.split(' ')[0]}.`:page}</h1><p className="muted">A calm space to build momentum, one session at a time.</p>{error&&<div className="error" role="alert">{error}<button className="text-button" onClick={()=>setError('')}>Dismiss</button></div>}<fieldset disabled={busy} className="workspace-content">{page==='Profile & Settings'?<Settings user={data.user} act={act}/>:page==='Focus Rooms'?<Rooms data={data} act={act} onNavigate={setPage}/>:page==='Focus Timer'?<><Timer data={data} act={act}/><SessionHistory sessions={data.sessions}/></>:<section className="card welcome"><span className="eyebrow">YOUR NEXT CHAPTER</span><h2>Small steps. Meaningful progress.</h2><p>Your local account is ready. Set your focus rhythm to get started.</p><button className="primary" onClick={()=>setPage('Profile & Settings')}>Personalize your workspace →</button></section>}</fieldset></div></main></div>
}
