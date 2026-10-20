use crate::db::{Database, DbResult, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

pub fn event(
    conn: &Connection,
    user: i64,
    session: Option<i64>,
    kind: &str,
    details: Value,
    now: i64,
) -> Result<()> {
    conn.execute("INSERT INTO focus_events(user_id,session_id,event_type,details,created_at) VALUES(?,?,?,?,?)",params![user,session,kind,details.to_string(),now]).safe()?;
    Ok(())
}

impl Database {
    pub fn start_session(
        &mut self,
        minutes: i64,
        kind: &str,
        room: Option<i64>,
        now: i64,
    ) -> Result<()> {
        let user = self.user()?;
        if !(1..=180).contains(&minutes) || !["focus", "short_break", "long_break"].contains(&kind)
        {
            return Err("Choose a session between 1 and 180 minutes.".into());
        }
        if kind != "focus" && room.is_some() {
            return Err("Only focus sessions can be linked to rooms.".into());
        }
        let tx = self.conn.transaction().safe()?;
        let active:i64=tx.query_row("SELECT COUNT(*) FROM focus_sessions WHERE user_id=? AND status IN ('running','paused')",[user],|r|r.get(0)).safe()?;
        if active > 0 {
            return Err("Finish or reset your current timer before starting another.".into());
        }
        if let Some(room) = room {
            let allowed:i64=tx.query_row("SELECT COUNT(*) FROM room_participants p JOIN rooms r ON r.id=p.room_id WHERE p.user_id=? AND p.room_id=? AND p.status='joined' AND r.status='open'",params![user,room],|r|r.get(0)).safe()?;
            if allowed == 0 {
                return Err("Join this open room before starting a room session.".into());
            }
        }
        let reward: i64 = tx
            .query_row("SELECT session_points FROM users WHERE id=?", [user], |r| {
                r.get(0)
            })
            .safe()?;
        tx.execute("INSERT INTO focus_sessions(user_id,room_id,kind,started_at,planned_seconds,running_since,status,reward_points) VALUES(?,?,?,?,?,?,'running',?)",params![user,room,kind,now,minutes*60,now,reward]).safe()?;
        let id = tx.last_insert_rowid();
        event(
            &tx,
            user,
            Some(id),
            "session_started",
            json!({"kind":kind,"minutes":minutes,"room_id":room}),
            now,
        )?;
        tx.commit().safe()
    }

    pub fn reconcile(&mut self, now: i64) -> Result<()> {
        self.user()?;
        // Reconcile elapsed timers for every local participant, including signed-out accounts.
        // This is internal clock-based completion; caller-supplied user IDs are never accepted.
        let due = {
            let mut stmt=self.conn.prepare("SELECT id,user_id FROM focus_sessions WHERE status='running' AND elapsed_ms+MAX(0,?-running_since)>=planned_seconds*1000").safe()?;
            let rows = stmt
                .query_map([now], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
                .safe()?
                .collect::<rusqlite::Result<Vec<_>>>()
                .safe()?;
            rows
        };
        for (id, user) in due {
            self.session_action_for(user, id, "reconcile", now)?;
        }
        Ok(())
    }

    pub fn session_action(&mut self, id: i64, op: &str, now: i64) -> Result<()> {
        self.session_action_for(self.user()?, id, op, now)
    }

    fn session_action_for(&mut self, user: i64, id: i64, op: &str, now: i64) -> Result<()> {
        if !["pause", "resume", "cancel", "complete", "reconcile"].contains(&op) {
            return Err("Unknown timer action.".into());
        }
        let tx = self.conn.transaction().safe()?;
        let row=tx.query_row("SELECT status,elapsed_ms,running_since,planned_seconds,kind,reward_points FROM focus_sessions WHERE id=? AND user_id=?",params![id,user],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,i64>(3)?,r.get::<_,String>(4)?,r.get::<_,i64>(5)?))).optional().safe()?.ok_or("Session not found in your account.")?;
        let (status, elapsed, since, planned, kind, reward) = row;
        if status == "completed" || status == "cancelled" {
            return Ok(());
        }
        let total = (elapsed + since.map(|t| (now - t).max(0)).unwrap_or(0)).min(planned * 1000);
        // Only the native clock can complete a session; retries never create another reward.
        if status == "running" && total >= planned * 1000 {
            let end = since.unwrap_or(now) + (planned * 1000 - elapsed).max(0);
            let points = if kind == "focus" { reward } else { 0 };
            tx.execute("UPDATE focus_sessions SET status='completed',elapsed_ms=?,actual_seconds=?,ended_at=?,running_since=NULL,points_awarded=? WHERE id=?",params![planned*1000,planned,end,points,id]).safe()?;
            event(
                &tx,
                user,
                Some(id),
                "session_completed",
                json!({"kind":kind,"minutes":planned/60,"points":points}),
                end,
            )?;
            if points > 0 {
                tx.execute("INSERT INTO point_transactions(user_id,session_id,amount,reason,idempotency_key,created_at) VALUES(?,?,?,'Focus session completed',?,?)",params![user,id,points,format!("session:{id}"),end]).safe()?;
                event(
                    &tx,
                    user,
                    Some(id),
                    "reward_created",
                    json!({"points":points}),
                    end,
                )?;
            }
            if kind == "focus" {
                crate::rewards::unlock(&tx, user, end)?;
            }
        } else {
            match op {
                "pause" if status == "running" => {
                    tx.execute("UPDATE focus_sessions SET status='paused',elapsed_ms=?,actual_seconds=?,running_since=NULL WHERE id=?",params![total,total/1000,id]).safe()?;
                    event(
                        &tx,
                        user,
                        Some(id),
                        "session_paused",
                        json!({"kind":kind}),
                        now,
                    )?;
                }
                "resume" if status == "paused" => {
                    tx.execute(
                        "UPDATE focus_sessions SET status='running',running_since=? WHERE id=?",
                        params![now, id],
                    )
                    .safe()?;
                    event(
                        &tx,
                        user,
                        Some(id),
                        "session_resumed",
                        json!({"kind":kind}),
                        now,
                    )?;
                }
                "cancel" => {
                    tx.execute("UPDATE focus_sessions SET status='cancelled',elapsed_ms=?,actual_seconds=?,ended_at=?,running_since=NULL WHERE id=?",params![total,total/1000,now,id]).safe()?;
                    event(
                        &tx,
                        user,
                        Some(id),
                        "session_cancelled",
                        json!({"kind":kind}),
                        now,
                    )?;
                }
                "complete" => {
                    return Err("This timer has not finished yet. Keep going or reset it.".into())
                }
                _ => {}
            }
        }
        tx.commit().safe()
    }

    pub fn session_list(&self, user: i64) -> Result<Vec<Value>> {
        let mut stmt=self.conn.prepare("SELECT s.id,s.room_id,s.kind,s.started_at,s.ended_at,s.planned_seconds,s.elapsed_ms,s.running_since,s.actual_seconds,s.status,s.points_awarded,r.name FROM focus_sessions s LEFT JOIN rooms r ON r.id=s.room_id WHERE s.user_id=? ORDER BY s.id DESC LIMIT 500").safe()?;
        let rows=stmt.query_map([user],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"room_id":r.get::<_,Option<i64>>(1)?,"kind":r.get::<_,String>(2)?,"started_at":r.get::<_,i64>(3)?,"ended_at":r.get::<_,Option<i64>>(4)?,"planned_seconds":r.get::<_,i64>(5)?,"elapsed_ms":r.get::<_,i64>(6)?,"running_since":r.get::<_,Option<i64>>(7)?,"actual_seconds":r.get::<_,i64>(8)?,"status":r.get::<_,String>(9)?,"points_awarded":r.get::<_,i64>(10)?,"room_name":r.get::<_,Option<String>>(11)?}))).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Request;
    use std::path::Path;
    pub fn account() -> Database {
        let mut db = Database::open(Path::new(":memory:")).unwrap();
        db.dispatch_at(
            Request::Register {
                email: "focus@example.com".into(),
                password: "focus password".into(),
                display_name: "Focus".into(),
            },
            1_000_000,
        )
        .unwrap();
        db
    }
    #[test]
    fn pause_resume_completion_is_atomic_and_idempotent() {
        let mut db = account();
        let t = 1_000_000;
        db.start_session(1, "focus", None, t).unwrap();
        assert!(db.start_session(1, "focus", None, t).is_err());
        assert!(db.session_action(1, "complete", t + 1000).is_err());
        db.session_action(1, "pause", t + 20_000).unwrap();
        db.session_action(1, "pause", t + 21_000).unwrap();
        db.reconcile(t + 200_000).unwrap();
        assert_eq!(
            db.snapshot(t).unwrap()["active_session"]["elapsed_ms"],
            20_000
        );
        db.session_action(1, "resume", t + 200_000).unwrap();
        db.reconcile(t + 240_000).unwrap();
        db.session_action(1, "complete", t + 300_000).unwrap();
        let s = db.snapshot(t).unwrap();
        assert_eq!(s["points"], 10);
        assert_eq!(s["transactions"].as_array().unwrap().len(), 1);
        assert_eq!(s["sessions"][0]["actual_seconds"], 60);
        assert_eq!(s["sessions"][0]["ended_at"], t + 240_000);
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT COUNT(*) FROM focus_events WHERE event_type='session_paused'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
    }
    #[test]
    fn cancellation_and_breaks_do_not_reward() {
        let mut db = account();
        db.start_session(1, "focus", None, 0).unwrap();
        db.session_action(1, "cancel", 30_000).unwrap();
        db.session_action(1, "complete", 70_000).unwrap();
        db.start_session(1, "short_break", None, 80_000).unwrap();
        db.reconcile(140_000).unwrap();
        assert_eq!(db.snapshot(140_000).unwrap()["points"], 0);
    }
    #[test]
    fn ownership_and_constraints() {
        let mut db = account();
        db.start_session(1, "focus", None, 0).unwrap();
        db.user_id = Some(999);
        assert!(db.session_action(1, "cancel", 1000).is_err());
        assert!(db.conn.execute("INSERT INTO room_participants(room_id,user_id,joined_at,status) VALUES(123,999,0,'joined')",[]).is_err());
    }
}
