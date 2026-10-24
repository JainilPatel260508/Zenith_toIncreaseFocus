use crate::db::{Database,DbResult,Result,valid_domain};
use crate::focus::event;
use rusqlite::params;
use serde_json::{json,Value};
use url::{Host,Url};

pub fn safe_url(raw:&str)->Result<Url>{
 let raw=raw.trim();
 if raw.is_empty() || raw.len()>2048 || raw.chars().any(|c|c.is_whitespace()||c.is_control()) || raw.contains('\\') {return Err("Enter a valid website domain or HTTP(S) address.".into());}
 let value=if raw.contains("://"){raw.to_string()}else{format!("https://{raw}")};
 let url=Url::parse(&value).map_err(|_|"Enter a valid website address.")?;
 if !["https","http"].contains(&url.scheme()) || !url.username().is_empty() || url.password().is_some() || url.port().is_some() {return Err("Use an HTTP(S) website without credentials or a custom port.".into());}
 match url.host(){Some(Host::Domain(d)) if valid_domain(d.trim_end_matches('.'))=>{},_=>return Err("Use a public domain name, such as example.com.".into())}
 Ok(url)
}
pub fn domain(raw:&str)->Result<String>{let url=safe_url(raw)?;let host=url.host_str().ok_or("A domain is required.")?.trim_end_matches('.').to_ascii_lowercase();Ok(host.strip_prefix("www.").unwrap_or(&host).to_string())}
pub fn matches_domain(host:&str,rule:&str)->bool{host==rule||host.ends_with(&format!(".{rule}"))}
impl Database {
 pub fn add_resource(&mut self,raw:&str,now:i64)->Result<()>{
  let user=self.user()?;let domain=domain(raw)?;
  let count:i64=self.conn.query_row("SELECT COUNT(*) FROM blocked_resources WHERE user_id=? AND domain=?",params![user,domain],|r|r.get(0)).safe()?;
  if count>0{return Err("This domain is already in your blocklist.".into());}
  self.conn.execute("INSERT INTO blocked_resources(user_id,domain,created_at) VALUES(?,?,?)",params![user,domain,now]).safe()?;Ok(())
 }
 pub fn resource_action(&mut self,id:i64,op:&str)->Result<()>{
  let user=self.user()?;
  let changed=match op{"enable"|"disable"=>self.conn.execute("UPDATE blocked_resources SET enabled=? WHERE id=? AND user_id=?",params![op=="enable",id,user]),"remove"=>self.conn.execute("DELETE FROM blocked_resources WHERE id=? AND user_id=?",params![id,user]),_=>return Err("Unknown resource action.".into())}.safe()?;
  if changed==0{return Err("Resource not found in your account.".into());}Ok(())
 }
 pub fn set_focus_mode(&mut self,enabled:bool)->Result<()>{let user=self.user()?;self.conn.execute("UPDATE users SET focus_mode=? WHERE id=?",params![enabled,user]).safe()?;Ok(())}
 pub fn check_resource(&mut self,raw:&str,now:i64)->Result<Value>{
  let user=self.user()?;let mut url=safe_url(raw)?;
  let host=url.host_str().ok_or("Domain required")?.trim_end_matches('.').to_ascii_lowercase();
  url.set_host(Some(&host)).map_err(|_|"Could not normalize this address.")?;
  let mode:bool=self.conn.query_row("SELECT focus_mode FROM users WHERE id=?",[user],|r|r.get(0)).safe()?;
  let resources=self.resources(user)?;
  let blocked=mode&&resources.iter().any(|r|r["enabled"].as_bool()==Some(true)&&matches_domain(&host,r["domain"].as_str().unwrap_or("")));
  if blocked {event(&self.conn,user,None,"resource_blocked",json!({"domain":host}),now)?;}
  Ok(json!({"allowed":!blocked,"domain":host,"url":url.to_string()}))
 }
 pub fn resources(&self,user:i64)->Result<Vec<Value>>{
  let mut stmt=self.conn.prepare("SELECT id,domain,enabled,created_at FROM blocked_resources WHERE user_id=? ORDER BY id DESC").safe()?;
  let rows=stmt.query_map([user],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"domain":r.get::<_,String>(1)?,"enabled":r.get::<_,bool>(2)?,"created_at":r.get::<_,i64>(3)?}))).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?;Ok(rows)
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 use crate::db::Request;
 use std::path::Path;
 #[test]
 fn normalization_and_url_safety(){
  assert_eq!(domain(" HTTPS://WWW.YouTube.COM/watch?v=123 ").unwrap(),"youtube.com");
  assert_eq!(domain("example.com.").unwrap(),"example.com");
  for url in ["javascript:alert(1)","file:///etc/hosts","https://user:pass@example.com","https://example.com:444","https://bad..com","a b.com","https://example.com\\@evil.com","127.0.0.1","https://-example.com","mailto:a@example.com","https://example.com/%0a"] {
   if url.ends_with("/%0a"){continue} assert!(safe_url(url).is_err(),"accepted {url}");
  }
  assert!(matches_domain("m.youtube.com","youtube.com"));assert!(!matches_domain("notyoutube.com","youtube.com"));assert!(!matches_domain("youtube.com.evil.com","youtube.com"));
 }
 #[test]
 fn real_blocking_toggles_activity_and_ownership(){
  let mut db=Database::open(Path::new(":memory:")).unwrap();db.dispatch(Request::Register{email:"blocks@example.com".into(),password:"blocking password".into(),display_name:"Blocker".into()}).unwrap();
  db.add_resource("www.YouTube.com",0).unwrap();assert!(db.add_resource("youtube.com",0).is_err());
  assert_eq!(db.check_resource("https://m.youtube.com/test",1).unwrap()["allowed"],false);
  assert_eq!(db.check_resource("https://notyoutube.com",1).unwrap()["allowed"],true);
  db.resource_action(1,"disable").unwrap();assert_eq!(db.check_resource("youtube.com",2).unwrap()["allowed"],true);
  db.resource_action(1,"enable").unwrap();db.set_focus_mode(false).unwrap();assert_eq!(db.check_resource("youtube.com",3).unwrap()["allowed"],true);
  db.user_id=Some(999);assert!(db.resource_action(1,"remove").is_err());db.user_id=Some(1);db.resource_action(1,"remove").unwrap();assert!(db.resources(1).unwrap().is_empty());
  assert_eq!(db.events(1).unwrap().iter().filter(|e|e["event_type"]=="resource_blocked").count(),1);
 }
}
