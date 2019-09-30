# Distance Leaderboard Scores

This repository consists of three projects:
* `test_app`: a Rust project that acts like the game [Distance](https://store.steampowered.com/app/233610/Distance/) using [steamworks-rs](https://github.com/Seeker14491/steamworks-rs), downloads the leaderboards to an sqlite3 database, and calculates scores.
* `OfficialLevelsRetriever`: A C# [Spectrum](https://github.com/Ciastex/Spectrum) plugin for Distance that prints out all official level file names for use in `test_app`.
* `Browser`: A basic Node.js server that allows browsing of the leaderboard data. This is live at http://leaderboard.distance.rocks!

---

These projects were quickly hacked together for fun. Most of their internals are messy, incomplete, and use hardcoded values. (see: the name "test_app" for the leaderboard downloader 😛) I'm hoping to improve these projects with time once the fundamentals are solid.

---

## Score Calculation

Player score is meant to be a measure of player skill. Player scores are calculated in steps:

1. For every level a player has played, get their placement on that levels' leaderboard
2. For every level a player has played, calculated `leaderboard_entries_count - player_placement + 1`. This is the player's score on that level. Examples:
    * 1 out of 100: `100 - 1 + 1 = 100`
    * 1 out of 27: `27 - 1 + 1 = 27`
    * 7 out of 100: `100 - 7 + 1 = 94`
    * last out of 306: `306 - 306 + 1 = 1`
3. Sum up all of a player's individual level scores. This is the player's absolute score.

Level difficulties are meant to be a measure of the skill necessary to beat a level in a reasonable time. Level difficulties are calculated in steps:

1. Calculate absolute scores for every player.
2. For a given level, sum up the absolute scores of all the players who have finished the level.
3. Calculate the difficulty as `absolute_score_sum/leaderboard_entries_count`. This is the level difficulty. It is effectively the average score of all players who have finished the level.
4. Multiply the above by `min(leaderboard_entires_count, 30)`. This gives levels with fewer than 30 entries a lower difficulty score.

---

## Player Scoring Flaws

Player scoring has some flaws:

1. Finishing hard levels does not grant more score points.
2. Small millisecond-level optimizations near the top of the leaderboard do not grant many points despite being difficult.
3. Finishing popular levels grant significantly more score points. For example, finishing [Static Fire Signal](http://leaderboard.distance.rocks/level/Static%20Fire%20Signal_1_stable) gives significantly more score points (19120 at maximum) that finishing [Paradise](http://leaderboard.distance.rocks/level/inferno%20reverse_1_76561198033370644_stable), (44 at maximum) despite Paradise being more difficult.
4. Finishing many levels at mid-leaderboard placement can give you a significantly higher skill score even at a lower actual skill level.

## Difficulty Scoring Flaws

Difficulty scoring has some flaws:

1. If any low-skill-score players finishes a hard level, it brings the level difficulty down.
2. without `min(30, ...)`: Easy levels finished by only one or two high-skill players have high difficulty scores
3. with `min(30, ...)`: Hard levels without enough leaderboard entries have lower difficulty scores.

---

## Player Scoring Solutions

Player scoring has a few possible fixes with their own pros and cons:

* Calculate scores using time-from-first
    * Fixes 1 and 3, but all other issues remain
* Calculate scores using a variant of `log(time_from_first)` where score changes significantly near the top of the leaderboard and very little near the bottom
    * Fixes 1, 2, and 3
    * Causes leaderboard entry count to matter *more* since all levels are similarly-scored. This could maybe be fixed by taking the average individual level score instead of a sum of all scores.
* Calculate scores by tallying up individual *player vs player* `wins - losses`. This generates a score value for every player that you're higher than or lower than on a level's leaderboard, where higher gives you `+1` and lower gives you `-1`. You absolute score would be the sum of every win (`+1` per level per player) vs loss (`-1` per level per player)
    * This is hard to analyze mentally as there are many variables
    * On a single level, this is the same as the current scoring - 1.
    * On multiple levels, this can negate the advantage of beating extra levels at a mid-leaderboard rank since the higher-ranking players negate your score.
    * This *might* solve issue 4, but leaves all other issues in play.
    * With a modification, this might work well to combat issue 3: take each individual `playerA_vs_playerB` score and apply an operation that increases the score as you beat the player more times like `playerA_vs_playerB^2`. This makes the 19,000 people you beat nearly worthless vs the same 100 people you beat on most maps.

## Difficulty Scoring Solutions

Issues 2 and 3 must be reasonable balanced against each other but neither can be fully solved.

A simple fix to issue 1 is to scale the effect of a player's score with their placement, such that placing first has a high effect and placing last has near no effect.