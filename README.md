# Zenith — A Little More Focus, Every Day

Zenith is a calm, local-first desktop productivity application built for macOS, designed to help you cultivate deep focus, track meaningful momentum, and minimize digital distractions.

Built natively using [Tauri v2](https://tauri.app/), [React 19](https://react.dev/), and embedded [SQLite](https://sqlite.org/), Zenith keeps all your sessions, habits, analytics, and achievements entirely on your Mac.

---

## Features

- **Persistent Focus Timer**: Custom focus intervals (Pomodoro or bespoke durations) with suspension-safe elapsed time tracking that accounts for macOS sleep and lock states.
- **Local Focus Rooms**: Create and participate in local collaborative focus rooms with real-time presence counts and shared intention.
- **Activity & Analytics Timeline**: Track daily streaks, focus minutes, completion ratios, and historical session timelines.
- **Resource & Website Blocker**: Enforce focused work by maintaining a curated domain blocklist with controlled native launch verification.
- **Achievements & Reward Ledger**: Earn points through completed focus sessions with an idempotent transaction ledger and milestone achievements.
- **Local & Private by Design**: Local account authentication hashed with Argon2, encrypted settings, and zero telemetry or cloud lock-in.

---

## Architecture & Tech Stack

- **Frontend**: React 19, Vite, Prettier, Oxlint
- **Desktop Host**: Tauri v2, Rust 2021 edition
- **Data Persistence**: Bundled SQLite 3 (`rusqlite`) with schema migrations and foreign key constraints
- **Security**: Argon2id password hashing, isolated IPC commands, strict Content Security Policy (CSP)
- **Audio Feedback**: Web Audio API completion chime synthesis

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+ recommended)
- [Rust toolchain](https://www.rust-lang.org/) (Cargo & rustc)
- Xcode Command Line Tools (on macOS: `xcode-select --install`)

### Installation

Clone the repository and install frontend dependencies:

```bash
git clone https://github.com/JainilPatel260508/Zenith_toIncreaseFocus.git
cd Zenith_toIncreaseFocus
npm install
```

### Development

Run the frontend in development mode:

```bash
npm run dev
```

Launch the native Tauri desktop application:

```bash
npm run desktop
```

---

## Verification & Testing

Zenith includes end-to-end automated tests covering both frontend logic and the native Rust/SQLite database layer:

```bash
# Run all automated tests (Node.js assertions + Rust native unit & integration tests)
npm test

# Run Oxlint linter
npm run lint

# Run Vite production build check
npm run build

# Run interactive smoke verification bridge
npm run test:ui
```

---

## Packaging & Distribution

To package Zenith into a signed macOS application and DMG installer:

```bash
# Build native macOS app bundle and DMG
npm run package:mac

# Verify Apple UDIF disk image signature and size constraints (< 50MB)
npm run verify:dmg
```

---

## License

Private & Personal Workspace · Zenith Productivity
