use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use rand_core::OsRng;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{path::Path, time::{Duration, Instant}};

pub type Result<T> = std::result::Result<T, String>;
pub trait DbResult<T> { fn safe(self) -> Result<T>; }
impl<T> DbResult<T> for rusqlite::Result<T> {
    fn safe(self) -> Result<T> { self.map_err(|e| { eprintln!("Zenith database: {e}"); "The local database could not complete this action. Please try again.".into() }) }
}

#[derive(Deserialize, Debug)]
#[serde(tag="action", rename_all="snake_case", deny_unknown_fields)]
pub enum Request {
    Bootstrap,
    Register { email: String, password: String, display_name: String },
    Login { email: String, password: String },
    Logout,
    Snapshot,
    StartSession { minutes: i64, kind: String, room_id: Option<i64> },
    SessionAction { session_id: i64, operation: String },
    SaveSettings { display_name: String, email: String, focus_minutes: i64, short_break: i64, long_break: i64, theme: String, notifications: bool, session_points: i64 },
}

pub struct Database {
    pub conn: Connection,
    pub user_id: Option<i64>,
    failures: u32,
    locked_until: Option<Instant>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = Connection::open(path).safe()?;
        conn.busy_timeout(Duration::from_secs(5)).safe()?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;").safe()?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).safe()?;
        if version > 1 { return Err("This database was created by a newer Zenith version.".into()); }
        if version < 1 {
            let tx = conn.transaction().safe()?;
            tx.execute_batch(include_str!("../migrations/001_initial.sql")).safe()?;
            tx.execute_batch("PRAGMA user_version=1").safe()?;
            tx.commit().safe()?;
        }
        Ok(Self { conn, user_id: None, failures: 0, locked_until: None })
    }

    pub fn user(&self) -> Result<i64> { self.user_id.ok_or_else(|| "Please sign in to continue.".into()) }

    pub fn dispatch(&mut self, req: Request) -> Result<Value> {
        self.dispatch_at(req, chrono::Utc::now().timestamp_millis())
    }

    pub fn dispatch_at(&mut self, req: Request, now: i64) -> Result<Value> {
        match req {
            Request::Bootstrap => {
                let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).safe()?;
                return Ok(json!({"has_accounts": count > 0, "authenticated": self.user_id.is_some()}));
            },
            Request::Register { email, password, display_name } => {
                let email = validate_email(&email)?;
                let name = validate_name(&display_name)?;
                if password.len() < 10 || password.len() > 128 { return Err("Use a password between 10 and 128 characters.".into()); }
                if self.conn.query_row("SELECT id FROM users WHERE email=?", [&email], |r| r.get::<_,i64>(0)).optional().safe()?.is_some() { return Err("An account with this email already exists on this Mac.".into()); }
                let salt = SaltString::generate(&mut OsRng);
                let hash = Argon2::default().hash_password(password.as_bytes(), &salt).map_err(|_| "Could not securely create your account.")?.to_string();
                self.conn.execute("INSERT INTO users(email,password_hash,display_name,created_at) VALUES(?,?,?,?)", params![email,hash,name,now]).safe()?;
                self.user_id = Some(self.conn.last_insert_rowid());
                self.failures=0;
            },
            Request::Login { email, password } => {
                if self.locked_until.is_some_and(|until| until > Instant::now()) { return Err("Too many attempts. Please wait 30 seconds.".into()); }
                if password.len() > 128 { return Err("Email or password is incorrect.".into()); }
                let email = validate_email(&email)?;
                let found = self.conn.query_row("SELECT id,password_hash FROM users WHERE email=?", [&email], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?))).optional().safe()?;
                let verified = found.as_ref().is_some_and(|(_,hash)| PasswordHash::new(hash).is_ok_and(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok()));
                if !verified {
                    self.failures += 1;
                    if self.failures >= 5 { self.locked_until = Some(Instant::now()+Duration::from_secs(30)); self.failures=0; }
                    return Err("Email or password is incorrect.".into());
                }
                self.user_id=found.map(|(id,_)|id); self.failures=0; self.locked_until=None;
            },
            Request::Logout => { self.user_id=None; return Ok(json!({"ok":true})); },
            Request::Snapshot => {},
            Request::StartSession { minutes, kind, room_id } => { self.reconcile(now)?; self.start_session(minutes, &kind, room_id, now)?; },
            Request::SessionAction { session_id, operation } => { self.session_action(session_id, &operation, now)?; },
            Request::SaveSettings { display_name, email, focus_minutes, short_break, long_break, theme, notifications, session_points } => {
                let id=self.user()?;
                let name=validate_name(&display_name)?; let email=validate_email(&email)?;
                if !(1..=180).contains(&focus_minutes) || !(1..=60).contains(&short_break) || !(1..=120).contains(&long_break) || !(1..=100).contains(&session_points) || !["dark","light"].contains(&theme.as_str()) { return Err("Check the duration, theme, and points settings.".into()); }
                let other: i64=self.conn.query_row("SELECT COUNT(*) FROM users WHERE email=? AND id<>?",params![email,id],|r|r.get(0)).safe()?;
                if other>0 {return Err("This email belongs to another local account.".into());}
                self.conn.execute("UPDATE users SET display_name=?,email=?,focus_minutes=?,short_break=?,long_break=?,theme=?,notifications=?,session_points=? WHERE id=?",params![name,email,focus_minutes,short_break,long_break,theme,notifications,session_points,id]).safe()?;
            },
        }
        self.reconcile(now)?;
        self.snapshot(now)
    }

    pub fn snapshot(&self, now: i64) -> Result<Value> {
        let id=self.user()?;
        let user=self.conn.query_row("SELECT id,email,display_name,focus_minutes,short_break,long_break,theme,notifications,focus_mode,session_points,created_at FROM users WHERE id=?",[id],|r| Ok(json!({
            "id":r.get::<_,i64>(0)?,"email":r.get::<_,String>(1)?,"display_name":r.get::<_,String>(2)?,"focus_minutes":r.get::<_,i64>(3)?,"short_break":r.get::<_,i64>(4)?,"long_break":r.get::<_,i64>(5)?,"theme":r.get::<_,String>(6)?,"notifications":r.get::<_,bool>(7)?,"focus_mode":r.get::<_,bool>(8)?,"session_points":r.get::<_,i64>(9)?,"created_at":r.get::<_,i64>(10)?
        }))).safe()?;
        let sessions=self.session_list(id)?;
        let active=sessions.iter().find(|s| s["status"]=="running" || s["status"]=="paused").cloned();
        let points:i64=self.conn.query_row("SELECT COALESCE(SUM(amount),0) FROM point_transactions WHERE user_id=?",[id],|r|r.get(0)).safe()?;
        let mut stmt=self.conn.prepare("SELECT amount,reason,created_at,session_id FROM point_transactions WHERE user_id=? ORDER BY id DESC LIMIT 100").safe()?;
        let transactions=stmt.query_map([id],|r|Ok(json!({"amount":r.get::<_,i64>(0)?,"reason":r.get::<_,String>(1)?,"created_at":r.get::<_,i64>(2)?,"session_id":r.get::<_,Option<i64>>(3)?}))).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?;
        Ok(json!({"user":user,"server_now":now,"sessions":sessions,"active_session":active,"points":points,"transactions":transactions}))
    }
}

pub fn validate_name(s:&str)->Result<String>{ let s=s.trim(); if s.is_empty() || s.chars().count()>60 || s.chars().any(char::is_control) {Err("Enter a name between 1 and 60 characters.".into())}else{Ok(s.to_string())} }
pub fn validate_email(s:&str)->Result<String>{
    let s=s.trim().to_ascii_lowercase(); let parts:Vec<_>=s.split('@').collect();
    if s.len()>254 || !s.is_ascii() || parts.len()!=2 || parts[0].is_empty() || parts[0].len()>64 || parts[0].starts_with('.') || parts[0].ends_with('.') || parts[0].contains("..") || !parts[0].chars().all(|c|c.is_ascii_alphanumeric() || ".!#$%&'*+-/=?^_`{|}~".contains(c)) || !valid_domain(parts.get(1).copied().unwrap_or("")) {return Err("Enter a valid email address.".into());}
    Ok(s)
}
pub fn valid_domain(s:&str)->bool{ s.len()<=253 && s.contains('.') && s.split('.').all(|l|!l.is_empty() && l.len()<=63 && !l.starts_with('-') && !l.ends_with('-') && l.chars().all(|c|c.is_ascii_alphanumeric()||c=='-')) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authentication_and_preferences() {
        let mut db=Database::open(Path::new(":memory:")).unwrap();
        assert!(db.dispatch(Request::Snapshot).is_err());
        assert!(validate_email("not-an-email").is_err());
        assert!(db.dispatch(Request::Register{email:"a@example.com".into(),password:"tiny".into(),display_name:"A".into()}).is_err());
        let register=|| Request::Register{email:"A@Example.com".into(),password:"correct horse battery".into(),display_name:"Asha".into()};
        let value=db.dispatch(register()).unwrap();
        assert_eq!(value["user"]["email"],"a@example.com");
        let hash:String=db.conn.query_row("SELECT password_hash FROM users",[],|r|r.get(0)).unwrap();
        assert!(hash.starts_with("$argon2id$")); assert!(!hash.contains("correct horse"));
        assert!(db.dispatch(register()).is_err());
        db.dispatch(Request::Logout).unwrap();
        assert!(db.dispatch(Request::Login{email:"a@example.com".into(),password:"incorrect".into()}).is_err());
        db.dispatch(Request::Login{email:"a@example.com".into(),password:"correct horse battery".into()}).unwrap();
        db.dispatch(Request::SaveSettings{email:"a@example.com".into(),display_name:"Asha P".into(),focus_minutes:50,short_break:10,long_break:20,theme:"light".into(),notifications:false,session_points:15}).unwrap();
        assert_eq!(db.snapshot(0).unwrap()["user"]["theme"],"light");
        assert_eq!(db.conn.query_row("PRAGMA foreign_keys",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    }
}
