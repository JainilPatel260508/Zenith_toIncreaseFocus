use crate::analytics::stats;
use crate::db::{Database, DbResult, Result};
use crate::focus::event;
use rusqlite::{params, Connection};
use serde_json::{json, Value};

fn progress(metric: &str, stats: &Value) -> i64 {
    match metric {
        "sessions" => stats["completed_sessions"].as_i64().unwrap_or(0),
        "minutes" => stats["total_seconds"].as_i64().unwrap_or(0) / 60,
        "streak" => stats["longest_streak"].as_i64().unwrap_or(0),
        _ => 0,
    }
}
pub fn unlock(conn: &Connection, user: i64, now: i64) -> Result<()> {
    let stats = stats(conn, user, now)?;
    let mut stmt=conn.prepare("SELECT id,name,metric,target FROM achievements WHERE id NOT IN (SELECT achievement_id FROM user_achievements WHERE user_id=?)").safe()?;
    let definitions = stmt
        .query_map([user], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .safe()?
        .collect::<rusqlite::Result<Vec<_>>>()
        .safe()?;
    for (id, name, metric, target) in definitions {
        if progress(&metric, &stats) >= target {
            let changed=conn.execute("INSERT INTO user_achievements(user_id,achievement_id,unlocked_at) VALUES(?,?,?) ON CONFLICT(user_id,achievement_id) DO NOTHING",params![user,id,now]).safe()?;
            if changed > 0 {
                event(
                    conn,
                    user,
                    None,
                    "achievement_unlocked",
                    json!({"achievement_id":id,"name":name}),
                    now,
                )?;
            }
        }
    }
    Ok(())
}
impl Database {
    pub fn achievements(&self, user: i64, now: i64) -> Result<Vec<Value>> {
        let stats = stats(&self.conn, user, now)?;
        let mut stmt=self.conn.prepare("SELECT a.id,a.code,a.name,a.description,a.metric,a.target,a.point_value,u.unlocked_at FROM achievements a LEFT JOIN user_achievements u ON u.achievement_id=a.id AND u.user_id=? ORDER BY a.id").safe()?;
        let rows=stmt.query_map([user],|r|{let metric:String=r.get(4)?;let target:i64=r.get(5)?;Ok(json!({"id":r.get::<_,i64>(0)?,"code":r.get::<_,String>(1)?,"name":r.get::<_,String>(2)?,"description":r.get::<_,String>(3)?,"metric":metric,"target":target,"point_value":r.get::<_,i64>(6)?,"unlocked_at":r.get::<_,Option<i64>>(7)?,"progress":progress(&metric,&stats).min(target)}))}).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?;
        Ok(rows)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Request;
    use std::path::Path;
    #[test]
    fn achievements_use_actual_totals_and_never_duplicate() {
        let mut db = Database::open(Path::new(":memory:")).unwrap();
        db.dispatch(Request::Register {
            email: "rewards@example.com".into(),
            password: "reward password".into(),
            display_name: "Rewards".into(),
        })
        .unwrap();
        let start = 1_700_000_000_000;
        for day in 0..7 {
            let now = start + day * 86_400_000;
            db.start_session(10, "focus", None, now).unwrap();
            db.reconcile(now + 600_000).unwrap();
        }
        let a = db.achievements(1, start + 7 * 86_400_000).unwrap();
        assert_eq!(a.iter().filter(|a| !a["unlocked_at"].is_null()).count(), 5);
        unlock(&db.conn, 1, start + 8 * 86_400_000).unwrap();
        assert_eq!(
            db.conn
                .query_row("SELECT COUNT(*) FROM user_achievements", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            5
        );
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT COUNT(*) FROM focus_events WHERE event_type='achievement_unlocked'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            5
        );
        let s = db.snapshot(start + 6 * 86_400_000 + 600_000).unwrap();
        assert_eq!(s["points"], 70);
        assert_eq!(s["stats"]["current_streak"], 7);
        assert_eq!(s["stats"]["today_seconds"], 600);
        assert_eq!(s["stats"]["week_seconds"], 4200);
    }
}
