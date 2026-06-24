# Spudbox — Project Plan & History

This is the fuller architectural/historical companion to `CLAUDE.md`. Read this when planning new work or picking the project back up after a context reset; read `CLAUDE.md` for day-to-day operational gotchas.

## Origin

The user (Luke) wanted a custom Linux music player as a nicer alternative to Clementine, inspired by MusicBee (which he likes on Windows but doesn't use, since he's ~99% on Linux). He explicitly didn't want to write Python. Primary emphasis: a genuinely nice, modern, custom UI. Hard functional requirements: real library browsing (not drag-and-drop a folder each session) and correct playback of hi-res files including FLAC.

Confirmed scope, established early via direct Q&A (do not re-litigate these without a reason):
- **Linux-only.** Not a cross-platform requirement right now.
- **Standard (non-exclusive) audio output is fine.** No ASIO/exclusive-ALSA bit-perfect requirement — just correct hi-res decoding and gapless playback.
- **Library lives in local folders on disk**, predominantly FLAC with some MP3/AAC/WAV. Real collection: ~6,757 tracks at `/home/luke/Dropbox/Music/CD Rips` on the original dev machine (this exact path is hardcoded into one `#[ignore]`d Rust integration test — see `scanner/mod.rs` — which will skip on any other machine).
- **UI layout was explicitly left open** — not required to clone MusicBee's 3-pane layout. The team settled on layout "A": sidebar (artist/album browse) + main content (album grid ↔ track list) + a persistent transport bar.

## Stack decision and why

Tauri v2 + Rust + Svelte 5, chosen over Electron, Qt6/QML (`cxx-qt`), and GTK4+libadwaita, primarily because:
- Full web-tech UI flexibility (the stated #1 priority) at a fraction of Electron's bundle size, with native-speed Rust for scanning/DB/audio.
- GTK4+libadwaita actively resists custom skinning by design — a poor fit for "nice custom UI."
- Real precedent existed in this exact niche: **Musicat** (Tauri+Svelte, local-library player) proved the stack works, but its use of IndexedDB/Dexie hit a maintainer-documented perf bug at scale (1-1.7s playback-start delay at 24k tracks) that the maintainer attributed to that choice. This project uses **SQLite from day one** specifically to avoid repeating that mistake.
- Qt6/QML remains the credible fallback if WebKitGTK ever proves too unstable for the UI ambitions here — **fooyin** and **mtoc** (both real Qt6 players) prove that path works for this app category too.

Audio: pure-Rust `symphonia` (decode) + `rodio`/`cpal` (output), explicitly **not** GStreamer — confirmed sufficient for FLAC/MP3/AAC/WAV including real hi-res (24-bit/96-192kHz, verified against actual test files up to 96kHz/24-bit in the user's library). This keeps the Linux bundle (`AppImage`/`.deb`) free of GStreamer's system `.so` dependency story. (WebKitGTK itself does pull in `libgstreamer-*` transitively for its own HTML5 `<video>`/`<audio>` support — that's unrelated to and unavoidable alongside our own audio engine, confirmed via `ldd` on the built binary.)

## Phase status

| Phase | Scope | Status |
|---|---|---|
| 0 | Scaffolding (Tauri+Svelte+Rust project, ping command round-trip) | ✅ Done |
| 1 | Library scan + DB (walk, tag extraction, incremental rescan) | ✅ Done — verified against the real ~6.5k library: 12.4s full scan, 48ms no-op rescan, zero parse errors |
| 2 | Playback engine (single-track) + MPRIS | ✅ Done — hi-res FLAC verified by ear at 96kHz/24-bit |
| 3 | UI browsing (sidebar/grid/tracklist, album art, virtualization) | ✅ Done |
| 4 | Gapless + watcher + polish | ✅ Done, **with one deliberate scope change** — see below |
| 5 | Packaging (AppImage, .deb) | ✅ Done |
| 6 | Device sync (MP3 player/DAP sync) | ❌ Not started |

### Phase 4's scope change

The original plan called for a `notify`-based filesystem watcher with `PollWatcher` fallback for the Linux inotify watch-limit. The user explicitly asked to drop this in favor of a plain **scan on every app launch** — manual "Rescan Library" button remains for mid-session use. Rationale (user's call, not a fallback from difficulty): the existing incremental scan is already fast enough when nothing's changed (~48ms) that a live watcher's complexity wasn't worth it for this use case. If live watching is ever revisited, the original research is still valid: use `notify::RecommendedWatcher` per scan-root with a debounce (500ms–2s), falling back to `notify::PollWatcher` on `ENOSPC` from the kernel inotify watch limit, with a documented `fs.inotify.max_user_watches` sysctl fix surfaced to the user.

What Phase 4 *did* end up including, beyond the original plan:
- **Play history/stats**: `play_history` (append-only) and `track_stats` (play_count/last_played rollup) are written on every track start (not completion — see `CLAUDE.md`). No UI consumes this yet; it's groundwork for future "recently played"/"most played" views.
- **Session persistence**: a new `app_settings` key-value table stores volume and the last queue/index/position, checkpointed on track change, play/pause, volume change, and periodically (~10s) during uninterrupted playback. Restored on launch in **Paused** state — it deliberately never autoplays on open.

### Post-Phase-5 polish (real-world dogfooding)

Once the app was packaged and the user started using it day-to-day, several gaps and feature requests came up that weren't in the original plan:

- **"Add Music Folder" button** (native folder picker via `@tauri-apps/plugin-dialog`). This was a real bug, not a feature request: the original debug-page UI had a manual path text input that was the *only* way `library_add_root` was ever called; it got silently removed when the debug page was replaced with the real browsing UI, and nobody noticed until packaging the app for a second machine with an empty database and no way to point it at a library at all.
- **Now-playing → album navigation**: clicking the transport bar's art/title navigates to that track's album. Required threading `album_id` through `PlayableTrack`/`TrackInfo`/`PlaybackSnapshot`, which previously only carried the album *name*.
- **Sidebar search + expandable per-artist album lists**: a search box filtering by artist name or album title, with a caret per artist to expand/collapse its albums. Search matching normalizes away punctuation (curly vs. straight apostrophes, leading ellipses, etc. in tag metadata) before comparing — see `CLAUDE.md`.
- **Icon pass**: the transport controls, sidebar caret, and track-list back button were originally plain Unicode glyph characters (▶, ⏮, ▸, etc.), which render inconsistently across system fonts and looked visibly worse than the rest of the UI. Replaced with `@lucide/svelte` SVG icons.
- **Window sizing**: default window bumped to 1280×800 specifically so the album grid opens already sized for a clean 5-column layout; the grid's cards also now stretch to fill leftover row width (with square art) rather than leaving a dead gap at in-between window sizes.
- **Rename**: "music-player" → "Spudbox" (productName, Cargo package/lib name, npm package name, MPRIS display name — but **not** the Tauri `identifier`, deliberately, to avoid orphaning the existing app-data directory on machines that already had it installed).
- **Test suite**: 45 Rust unit tests (the queue model, the MPRIS seek helper, every `db::queries` module via an in-memory SQLite test connection, and scanner helpers) + 4 frontend vitest tests, plus a GitHub Actions CI workflow running both on PRs against `main`.

## Deferred items (explicitly out of scope, not forgotten)

These were flagged during planning or discovered during implementation and deliberately not pursued:

- **Various-artist/compilation album dedup** beyond the basic `(title, album_artist_id, year)` key. A real example surfaced in the user's actual library: one folder had ~99 duplicate-rip files of the same handful of tracks (e.g. `Track Name (7).flac`, `(8).flac`, ...), and another album was ripped with no artist tags at all (literally `27 Unknown Artist - Track 27.flac` as the filename). The scanner correctly reflects this messy real-world data rather than papering over it — cleanup, if wanted, would be a "find duplicates" feature, not a scanner fix.
- **Smart/rule-based playlists** — schema reserves an `is_smart` flag on `playlists`, no rule engine built.
- **Tag *writing*** — `lofty` supports it; nothing in this codebase uses that capability yet.
- **Crossfade** — distinct from gapless (which is implemented); not requested.
- **Winamp-style custom skinning** — CSS-custom-property theming is already the natural baseline (just how the UI is built), but true shaped-window/bitmap-skin support is a deferred stretch goal with a real, researched risk: Tauri v2 on Linux has documented 2026 GitHub issues of transparent/undecorated windows triggering Nvidia GBM/DMABUF compositor crashes. If ever picked up, plan a fallback to a plain rectangular skinned window.

## Phase 6 (not started): Device sync — research already done

Goal: detect a connected portable player (DAP) and sync missing tracks to it. Research findings, ready to use whenever this is picked up:

- **Detection**: use **UDisks2 over D-Bus** (the `udisks2` crate, or raw `zbus`), *not* `notify`-watching `/media`/`/run/media/$USER` — directory-watching can't reliably distinguish a removable DAP from any other filesystem change, and GNOME vs. KDE automount paths differ.
- **Device type is more mixed than expected**: plain/older DAPs and most Sony Walkman models use USB Mass Storage (the easy case — just a mounted filesystem). But many current Android-based hi-res DAPs (FiiO M-series, Hiby) primarily use **MTP**, the same protocol as Android phones. "Dedicated music player" does not reliably mean "mass storage."
- **MTP support in Rust is genuinely fragile right now**: `libmtp-rs` (FFI to the old `libmtp` C library) is alpha-quality; the newer pure-Rust `mtp-rs` is very young. The more reliable fallback is shelling out to `gio mount` and reading the resulting `/run/user/<uid>/gvfs/...` path (reuses what file managers already do).
- **Recommended scoping**: ship USB Mass Storage support first; treat MTP as a clearly-labeled secondary goal via the `gio`/gvfs shell-out path, not the native Rust MTP crates, given their current maturity.
- **Out of scope for v1 of this feature, when built**: on-the-fly transcoding during sync (copy original files as-is; skip with a warning if the device is out of space) and automatic deletion of device files no longer in the library (should be an explicit opt-in action, never automatic).

## Distribution

Repo: `git@github.com:lstebner/spudbox.git`, branch `main`. CI runs on PRs against `main` (not on direct pushes). Releases are built locally via `npm run tauri build -- --bundles deb,appimage` and are not yet automated in CI — there is no release workflow, version tagging convention, or auto-publish step. If that's wanted later, it would build on the existing `ci.yml` job structure.
