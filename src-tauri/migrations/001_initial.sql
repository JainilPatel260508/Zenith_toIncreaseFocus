CREATE TABLE users (
 id INTEGER PRIMARY KEY, email TEXT NOT NULL UNIQUE COLLATE NOCASE,
 password_hash TEXT NOT NULL, display_name TEXT NOT NULL,
 focus_minutes INTEGER NOT NULL DEFAULT 25 CHECK(focus_minutes BETWEEN 1 AND 180),
 short_break INTEGER NOT NULL DEFAULT 5 CHECK(short_break BETWEEN 1 AND 60),
 long_break INTEGER NOT NULL DEFAULT 15 CHECK(long_break BETWEEN 1 AND 120),
 theme TEXT NOT NULL DEFAULT 'dark' CHECK(theme IN ('dark','light')),
 notifications INTEGER NOT NULL DEFAULT 1 CHECK(notifications IN (0,1)),
 focus_mode INTEGER NOT NULL DEFAULT 1 CHECK(focus_mode IN (0,1)),
 session_points INTEGER NOT NULL DEFAULT 10 CHECK(session_points BETWEEN 1 AND 100),
 created_at INTEGER NOT NULL
);
CREATE TABLE rooms (
 id INTEGER PRIMARY KEY, owner_id INTEGER NOT NULL REFERENCES users(id),
 name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
 status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open','closed')),
 planned_minutes INTEGER NOT NULL CHECK(planned_minutes BETWEEN 1 AND 180), created_at INTEGER NOT NULL
);
CREATE TABLE room_participants (
 id INTEGER PRIMARY KEY, room_id INTEGER NOT NULL REFERENCES rooms(id),
 user_id INTEGER NOT NULL REFERENCES users(id), joined_at INTEGER NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('joined','left')), UNIQUE(room_id,user_id)
);
CREATE TABLE focus_sessions (
 id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id),
 room_id INTEGER REFERENCES rooms(id), kind TEXT NOT NULL DEFAULT 'focus' CHECK(kind IN ('focus','short_break','long_break')),
 started_at INTEGER NOT NULL, ended_at INTEGER,
 planned_seconds INTEGER NOT NULL CHECK(planned_seconds BETWEEN 60 AND 10800),
 elapsed_ms INTEGER NOT NULL DEFAULT 0 CHECK(elapsed_ms>=0), running_since INTEGER,
 actual_seconds INTEGER NOT NULL DEFAULT 0 CHECK(actual_seconds>=0),
 status TEXT NOT NULL CHECK(status IN ('running','paused','completed','cancelled')),
 reward_points INTEGER NOT NULL DEFAULT 10 CHECK(reward_points BETWEEN 1 AND 100),
 points_awarded INTEGER NOT NULL DEFAULT 0 CHECK(points_awarded>=0)
);
CREATE UNIQUE INDEX one_active_session_per_user ON focus_sessions(user_id) WHERE status IN ('running','paused');
CREATE INDEX sessions_user_date ON focus_sessions(user_id, ended_at);
CREATE TABLE focus_events (
 id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id),
 session_id INTEGER REFERENCES focus_sessions(id), event_type TEXT NOT NULL,
 details TEXT NOT NULL DEFAULT '{}', created_at INTEGER NOT NULL
);
CREATE INDEX events_user_date ON focus_events(user_id,created_at);
CREATE TABLE blocked_resources (
 id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), domain TEXT NOT NULL,
 enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)), created_at INTEGER NOT NULL, UNIQUE(user_id,domain)
);
CREATE TABLE achievements (
 id INTEGER PRIMARY KEY, code TEXT NOT NULL UNIQUE, name TEXT NOT NULL, description TEXT NOT NULL,
 metric TEXT NOT NULL CHECK(metric IN ('sessions','minutes','streak')), target INTEGER NOT NULL CHECK(target>0),
 point_value INTEGER NOT NULL DEFAULT 0 CHECK(point_value>=0)
);
CREATE TABLE user_achievements (
 id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), achievement_id INTEGER NOT NULL REFERENCES achievements(id),
 unlocked_at INTEGER NOT NULL, UNIQUE(user_id,achievement_id)
);
CREATE TABLE point_transactions (
 id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), session_id INTEGER REFERENCES focus_sessions(id),
 amount INTEGER NOT NULL CHECK(amount>0), reason TEXT NOT NULL, idempotency_key TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL
);
CREATE INDEX points_user ON point_transactions(user_id);
INSERT INTO achievements(code,name,description,metric,target) VALUES
 ('first_focus','The first step','Complete your first focus session.','sessions',1),
 ('five_sessions','Finding your rhythm','Complete five focus sessions.','sessions',5),
 ('sixty_minutes','An hour well spent','Accumulate 60 focused minutes.','minutes',60),
 ('three_day','Building momentum','Focus on three consecutive days.','streak',3),
 ('seven_day','A week of intention','Focus on seven consecutive days.','streak',7);
