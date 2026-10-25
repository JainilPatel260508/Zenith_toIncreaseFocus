use crate::db::{Database, DbResult, Result};
use chrono::{DateTime, Days, Local, NaiveDate};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub fn local_date(ms: i64) -> NaiveDate {
    DateTime::from_timestamp_millis(ms)
        .unwrap_or_default()
        .with_timezone(&Local)
        .date_naive()
}
pub fn streaks(days: &BTreeSet<NaiveDate>, today: NaiveDate) -> (i64, i64) {
    let mut longest = 0;
    let mut run = 0;
    let mut previous = None;
    for day in days {
        run = if previous.and_then(|p: NaiveDate| p.succ_opt()) == Some(*day) {
            run + 1
        } else {
            1
        };
        longest = longest.max(run);
        previous = Some(*day);
    }
    let mut current = 0;
    let mut cursor = if days.contains(&today) {
        Some(today)
    } else {
        today.pred_opt()
    };
    while let Some(day) = cursor {
        if !days.contains(&day) {
            break;
        }
        current += 1;
        cursor = day.pred_opt();
    }
    (current, longest)
}
pub fn stats(conn: &Connection, user: i64, now: i64) -> Result<Value> {
    let mut stmt=conn.prepare("SELECT ended_at,actual_seconds,status FROM focus_sessions WHERE user_id=? AND kind='focus' AND status IN ('completed','cancelled')").safe()?;
    let rows = stmt
        .query_map([user], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .safe()?
        .collect::<rusqlite::Result<Vec<_>>>()
        .safe()?;
    let today = local_date(now);
    let mut days = BTreeMap::<NaiveDate, i64>::new();
    let mut completed = 0;
    let mut total_seconds = 0;
    for (end, seconds, status) in &rows {
        if status == "completed" {
            completed += 1;
            total_seconds += seconds;
            *days.entry(local_date(*end)).or_default() += seconds;
        }
    }
    let dates = days.keys().copied().collect();
    let (current, longest) = streaks(&dates, today);
    let daily:Vec<_>=(0..7).rev().map(|offset|{let day=today.checked_sub_days(Days::new(offset)).unwrap_or(today);json!({"date":day.to_string(),"label":day.format("%a").to_string(),"seconds":days.get(&day).copied().unwrap_or(0),"today":offset==0})}).collect();
    let week_seconds: i64 = daily
        .iter()
        .map(|d| d["seconds"].as_i64().unwrap_or(0))
        .sum();
    let today_sessions = rows
        .iter()
        .filter(|(end, _, status)| status == "completed" && local_date(*end) == today)
        .count();
    Ok(
        json!({"completed_sessions":completed,"total_seconds":total_seconds,"today_seconds":days.get(&today).copied().unwrap_or(0),"today_sessions":today_sessions,"week_seconds":week_seconds,"current_streak":current,"longest_streak":longest,"completion_rate":if rows.is_empty(){0}else{completed*100/rows.len() as i64},"finished_sessions":rows.len(),"daily":daily}),
    )
}
impl Database {
    pub fn events(&self, user: i64) -> Result<Vec<Value>> {
        let mut stmt=self.conn.prepare("SELECT id,session_id,event_type,details,created_at FROM focus_events WHERE user_id=? ORDER BY created_at DESC,id DESC LIMIT 300").safe()?;
        let rows=stmt.query_map([user],|r|{let details:String=r.get(3)?;Ok(json!({"id":r.get::<_,i64>(0)?,"session_id":r.get::<_,Option<i64>>(1)?,"event_type":r.get::<_,String>(2)?,"details":serde_json::from_str::<Value>(&details).unwrap_or(json!({})),"created_at":r.get::<_,i64>(4)?}))}).safe()?.collect::<rusqlite::Result<Vec<_>>>().safe()?;
        Ok(rows)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streak_calendar_boundaries() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        let days = ["2025-12-30", "2025-12-31", "2026-01-01"]
            .map(|s| s.parse().unwrap())
            .into_iter()
            .collect();
        assert_eq!(streaks(&days, today), (3, 3));
        assert_eq!(streaks(&days, today.succ_opt().unwrap()), (0, 3));
        assert_eq!(streaks(&BTreeSet::new(), today), (0, 0));
    }
}
