CREATE TABLE IF NOT EXISTS players (
	steam_id TEXT NOT NULL PRIMARY KEY,
	score_sum INTEGER NOT NULL DEFAULT 0,
	score_count INTEGER NOT NULL DEFAULT 0,
	cached_display_name TEXT
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS levels (
	level_id TEXT NOT NULL PRIMARY KEY,
	score_sum INTEGER NOT NULL DEFAULT 0,
	score_count INTEGER NOT NULL DEFAULT 0,
	file_id TEXT,
	cached_display_name TEXT
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS steam_leaderboard (
	level_id TEXT NOT NULL,
	steam_id TEXT NOT NULL,
	time INTEGER NOT NULL,
	placement INTEGER NOT NULL DEFAULT 0,
	score INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(level_id, steam_id),
	FOREIGN KEY(level_id) REFERENCES levels(level_id),
	FOREIGN KEY(steam_id) REFERENCES players(steam_id)
) WITHOUT ROWID;


CREATE TRIGGER IF NOT EXISTS player_leaderboard_insert
	AFTER INSERT
	ON steam_leaderboard
BEGIN
	UPDATE players
	SET
		score_count = score_count + 1
	WHERE
		steam_id = NEW.steam_id;
	UPDATE levels
	SET
		score_count = score_count + 1
	WHERE
		level_id = NEW.level_id;
END;

CREATE TRIGGER IF NOT EXISTS player_leaderboard_delete
	AFTER DELETE
	ON steam_leaderboard
BEGIN
	UPDATE players
	SET
		score_count = score_count - 1,
		score_sum = score_sum - OLD.score
	WHERE
		steam_id = OLD.steam_id;
	UPDATE levels
	SET
		score_count = score_count + 1
	WHERE
		level_id = NEW.level_id;
END;

CREATE TRIGGER IF NOT EXISTS player_leaderboard_personal_score
	AFTER UPDATE
	ON steam_leaderboard
	WHEN OLD.score <> NEW.score
BEGIN
	UPDATE players
	SET
		score_sum = score_sum - OLD.score + NEW.score
	WHERE
		steam_id = NEW.steam_id;
END;

CREATE INDEX IF NOT EXISTS index_steam_leaderboard_placement ON steam_leaderboard(level_id, placement);

CREATE INDEX IF NOT EXISTS index_players_score_sum ON players(score_sum);

CREATE INDEX IF NOT EXISTS index_players_display_name ON players(cached_display_name);

CREATE INDEX IF NOT EXISTS index_levels_file_id ON levels(file_id);

CREATE INDEX IF NOT EXISTS index_levels_display_name ON levels(cached_display_name);