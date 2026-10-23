use crate::db::{Database,DbResult,Result,validate_name};
use crate::focus::event;
use rusqlite::{params,OptionalExtension};
use serde_json::{json,Value};

impl Database {
    pub fn create_room(&mut self,name:&str,description:&str,minutes:i64,now:i64)->Result<()> {
        let user=self.user()?; let name=validate_name(name)?;
        if description.chars().count()>500 || !(1..=180).contains(&minutes){return Err("Use a description under 500 characters and a duration of 1–180 minutes.".into());}
        let tx=self.conn.transaction().safe()?;
        tx.execute("INSERT INTO rooms(owner_id,name,description,planned_minutes,created_at) VALUES(?,?,?,?,?)",params![user,name,description.trim(),minutes,now]).safe()?;
        let room=tx.last_insert_rowid();
        tx.execute("INSERT INTO room_participants(room_id,user_id,joined_at,status) VALUES(?,?,?,'joined')",params![room,user,now]).safe()?;
        event(&tx,user,None,"room_joined",json!({"room_id":room,"room_name":name}),now)?;
        tx.commit().safe()
    }
    pub fn room_action(&mut self,room:i64,op:&str,target:Option<i64>,now:i64)->Result<()> {
        let user=self.user()?;
        let tx=self.conn.transaction().safe()?;
        let (owner,name,status)=tx.query_row("SELECT owner_id,name,status FROM rooms WHERE id=?",[room],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional().safe()?.ok_or("Room not found.")?;
        match op {
            "join"=>{
                if status!="open" {return Err("This room is closed.".into());}
                let changed=tx.execute("INSERT INTO room_participants(room_id,user_id,joined_at,status) VALUES(?,?,?,'joined') ON CONFLICT(room_id,user_id) DO UPDATE SET status='joined',joined_at=excluded.joined_at WHERE room_participants.status='left'",params![room,user,now]).safe()?;
                if changed>0 {event(&tx,user,None,"room_joined",json!({"room_id":room,"room_name":name}),now)?;}
            },
            "leave"|"remove"=>{
                let member=if op=="remove" {if owner!=user{return Err("Only the room owner can remove a participant.".into());}target.ok_or("Choose a participant.")?}else{user};
                let active:i64=tx.query_row("SELECT COUNT(*) FROM focus_sessions WHERE room_id=? AND user_id=? AND status IN ('running','paused')",params![room,member],|r|r.get(0)).safe()?;
                if active>0{return Err("Finish or reset this participant's room session before leaving or removing them.".into());}
                let changed=tx.execute("UPDATE room_participants SET status='left' WHERE room_id=? AND user_id=? AND status='joined'",params![room,member]).safe()?;
                if changed>0{event(&tx,member,None,"room_left",json!({"room_id":room,"room_name":name,"removed_by_owner":op=="remove"}),now)?;}
            },
            "close"|"reopen"=>{
                if owner!=user{return Err("Only the room owner can change its status.".into());}
                let active:i64=tx.query_row("SELECT COUNT(*) FROM focus_sessions WHERE room_id=? AND status IN ('running','paused')",[room],|r|r.get(0)).safe()?;
                if active>0{return Err("All participants must finish or reset their room sessions first.".into());}
                tx.execute("UPDATE rooms SET status=? WHERE id=?",params![if op=="close"{"closed"}else{"open"},room]).safe()?;
            },
            _=>return Err("Unknown room action.".into()),
        }
        tx.commit().safe()
    }
    pub fn room_list(&self)->Result<Vec<Value>> {
        let mut stmt=self.conn.prepare("SELECT r.id,r.owner_id,r.name,r.description,r.planned_minutes,r.status,u.display_name,(SELECT COUNT(*) FROM focus_sessions s WHERE s.room_id=r.id AND s.status='completed' AND s.kind='focus') FROM rooms r JOIN users u ON u.id=r.owner_id ORDER BY r.id DESC").safe()?;
        let mut rooms=stmt.query_map([],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"owner_id":r.get::<_,i64>(1)?,"name":r.get::<_,String>(2)?,"description":r.get::<_,String>(3)?,"planned_minutes":r.get::<_,i64>(4)?,"status":r.get::<_,String>(5)?,"owner_name":r.get::<_,String>(6)?,"completed_sessions":r.get::<_,i64>(7)?}))).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?;
        for room in &mut rooms {
            let id=room["id"].as_i64().unwrap_or(0);
            let mut stmt=self.conn.prepare("SELECT u.id,u.display_name,p.joined_at,s.status,s.planned_seconds,s.elapsed_ms,s.running_since FROM room_participants p JOIN users u ON u.id=p.user_id LEFT JOIN focus_sessions s ON s.user_id=p.user_id AND s.room_id=p.room_id AND s.status IN ('running','paused') WHERE p.room_id=? AND p.status='joined' ORDER BY p.joined_at").safe()?;
            room["participants"]=json!(stmt.query_map([id],|r|Ok(json!({"user_id":r.get::<_,i64>(0)?,"name":r.get::<_,String>(1)?,"joined_at":r.get::<_,i64>(2)?,"session_status":r.get::<_,Option<String>>(3)?,"planned_seconds":r.get::<_,Option<i64>>(4)?,"elapsed_ms":r.get::<_,Option<i64>>(5)?,"running_since":r.get::<_,Option<i64>>(6)?}))).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?);
        }
        Ok(rooms)
    }
}
#[cfg(test)]
mod tests {
 use super::*;
 use crate::db::Request;
 use std::path::Path;
 #[test]
 fn local_rooms_membership_and_ownership(){
    let mut db=Database::open(Path::new(":memory:")).unwrap();
    for (i,name) in [(1,"Asha"),(2,"Dev")] {db.dispatch(Request::Register{email:format!("user{i}@example.com"),password:"safe local password".into(),display_name:name.into()}).unwrap();}
    db.user_id=Some(1);db.create_room("Study","Local study",1,0).unwrap();
    db.user_id=Some(2);assert!(db.start_session(1,"focus",Some(1),0).is_err());
    db.room_action(1,"join",None,0).unwrap();db.room_action(1,"join",None,1).unwrap();
    assert_eq!(db.room_list().unwrap()[0]["participants"].as_array().unwrap().len(),2);
    assert!(db.room_action(1,"close",None,1).is_err());
    db.start_session(1,"focus",Some(1),0).unwrap();assert!(db.room_action(1,"leave",None,1000).is_err());
    db.reconcile(60000).unwrap();db.room_action(1,"leave",None,60001).unwrap();
    assert_eq!(db.snapshot(60001).unwrap()["sessions"][0]["room_id"],1);
    db.user_id=Some(1);db.room_action(1,"close",None,60002).unwrap();db.user_id=Some(2);assert!(db.room_action(1,"join",None,60003).is_err());
    db.user_id=Some(1);db.room_action(1,"reopen",None,60004).unwrap();db.room_action(1,"join",None,60005).unwrap();
    db.room_action(1,"remove",Some(2),60006).unwrap();
    assert_eq!(db.conn.query_row("SELECT COUNT(*) FROM room_participants",[],|r|r.get::<_,i64>(0)).unwrap(),2);
 }
}
