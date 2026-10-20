use crate::db::{Database, Request};
use std::path::PathBuf;

fn unique_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "zenith-test-{}-{}.sqlite3",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ))
}
fn register() -> Request {
    Request::Register {
        email: "persist@example.com".into(),
        password: "persistent password".into(),
        display_name: "Persistence".into(),
    }
}
fn login() -> Request {
    Request::Login {
        email: "persist@example.com".into(),
        password: "persistent password".into(),
    }
}
#[test]
fn migration_restart_paused_timer_and_reward_persistence() {
    let path = unique_path();
    let t = 1_700_000_000_000;
    {
        let mut db = Database::open(&path).unwrap();
        db.dispatch_at(register(), t).unwrap();
        db.create_room("Research", "Persistent local room", 1, t)
            .unwrap();
        db.add_resource("example.com", t).unwrap();
        db.start_session(1, "focus", Some(1), t).unwrap();
        db.session_action(1, "pause", t + 20_000).unwrap();
    }
    {
        let mut db = Database::open(&path).unwrap();
        assert!(db.dispatch_at(Request::Snapshot {}, t + 100_000).is_err());
        let snapshot = db.dispatch_at(login(), t + 100_000).unwrap();
        assert_eq!(snapshot["active_session"]["status"], "paused");
        assert_eq!(snapshot["active_session"]["elapsed_ms"], 20000);
        assert_eq!(
            snapshot["rooms"][0]["participants"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(snapshot["resources"][0]["domain"], "example.com");
        db.session_action(1, "resume", t + 100_000).unwrap();
    }
    {
        let mut db = Database::open(&path).unwrap();
        let snapshot = db.dispatch_at(login(), t + 180_000).unwrap();
        assert!(snapshot["active_session"].is_null());
        assert_eq!(snapshot["points"], 10);
        assert_eq!(snapshot["sessions"][0]["ended_at"], t + 140_000);
        assert_eq!(snapshot["achievements"][0]["name"], "The first step");
        assert!(!snapshot["achievements"][0]["unlocked_at"].is_null());
        db.dispatch_at(
            Request::SessionAction {
                session_id: 1,
                operation: "complete".into(),
            },
            t + 200_000,
        )
        .unwrap();
        assert_eq!(
            db.snapshot(t + 200_000).unwrap()["transactions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            db.conn
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        let violations: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(violations, 0);
        assert_eq!(
            db.conn
                .query_row("SELECT COUNT(*) FROM achievements", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            5
        );
    }
    std::fs::remove_file(path).unwrap();
}
#[test]
fn reward_failure_rolls_back_entire_completion() {
    let mut db = Database::open(std::path::Path::new(":memory:")).unwrap();
    db.dispatch_at(register(), 0).unwrap();
    db.start_session(1, "focus", None, 0).unwrap();
    db.conn.execute_batch("CREATE TRIGGER simulate_disk_failure BEFORE INSERT ON point_transactions BEGIN SELECT RAISE(ABORT,'simulated failure'); END;").unwrap();
    assert!(db.reconcile(60000).is_err());
    let s = db.snapshot(60000).unwrap();
    assert_eq!(s["active_session"]["status"], "running");
    assert_eq!(s["points"], 0);
    assert_eq!(s["events"].as_array().unwrap().len(), 1);
    db.conn
        .execute_batch("DROP TRIGGER simulate_disk_failure")
        .unwrap();
    db.reconcile(60000).unwrap();
    assert_eq!(db.snapshot(60000).unwrap()["points"], 10);
}
#[test]
fn reward_configuration_is_snapshotted_at_start() {
    let mut db = Database::open(std::path::Path::new(":memory:")).unwrap();
    db.dispatch_at(register(), 0).unwrap();
    db.start_session(1, "focus", None, 0).unwrap();
    db.dispatch_at(
        Request::SaveSettings {
            display_name: "Persistence".into(),
            email: "persist@example.com".into(),
            focus_minutes: 25,
            short_break: 5,
            long_break: 15,
            theme: "dark".into(),
            notifications: true,
            session_points: 42,
        },
        1000,
    )
    .unwrap();
    db.reconcile(60000).unwrap();
    assert_eq!(db.snapshot(60000).unwrap()["points"], 10);
    db.start_session(1, "focus", None, 70000).unwrap();
    db.reconcile(130000).unwrap();
    assert_eq!(db.snapshot(130000).unwrap()["points"], 52);
}
#[test]
fn invalid_ipc_inputs_are_rejected() {
    assert!(serde_json::from_str::<Request>(
        r#"{"action":"execute_sql","sql":"DROP TABLE users"}"#
    )
    .is_err());
    assert!(serde_json::from_str::<Request>(
        r#"{"action":"start_session","minutes":0.1,"kind":"focus","room_id":null}"#
    )
    .is_err());
    assert!(serde_json::from_str::<Request>(r#"{"action":"snapshot","user_id":999}"#).is_err());
}
