# Distance Leaderboard Scores

This repository consists of three projects:
* `evaluator`: a Rust project that acts like the game [Distance](https://store.steampowered.com/app/233610/Distance/) using [steamworks-rs](https://github.com/Seeker14491/steamworks-rs), downloads the leaderboards to an sqlite3 database, and calculates scores.
* `browser`: A basic Node.js server that allows browsing of the leaderboard data.
* `OfficialLevelsRetriever`: A C# [Spectrum](https://github.com/Ciastex/Spectrum) plugin for Distance that prints out all official level file names for use in `test_app`.

This repository also includes:
* A docker image for `evaluator` to run given a `.env` file with `PW_VNC`, `UN_STEAM` (username), and `PW_STEAM`
* A docker image for `browser` to run, reading the database file from the same mounted directory that `evaluator` writes to
* A docker image to build `evaluator` for Ubuntu 18.04
* Various shell scripts

---

These projects were quickly hacked together for fun. Most of their internals are messy, incomplete, and use hardcoded values.