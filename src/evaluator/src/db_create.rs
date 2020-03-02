
pub fn get_statements() -> Vec<&'static str> {
[
r#"
CREATE TABLE IF NOT EXISTS players (
	steam_id TEXT NOT NULL PRIMARY KEY,
	score_count INTEGER NOT NULL DEFAULT 0,
	evaluation_sum INTEGER NOT NULL DEFAULT 0,
	evaluation INTEGER NOT NULL DEFAULT 0,
	cached_display_name TEXT
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS levels (
	level_id TEXT NOT NULL PRIMARY KEY,
	score_count INTEGER NOT NULL DEFAULT 0,
	evaluation_sum INTEGER NOT NULL DEFAULT 0,
	evaluation INTEGER DEFAULT 0,
	file_id TEXT,
	cached_display_name TEXT
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS steam_leaderboard (
	level_id TEXT NOT NULL,
	steam_id TEXT NOT NULL,
	score INTEGER NOT NULL,
	placement INTEGER NOT NULL DEFAULT 0,
	evaluation INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(level_id, steam_id),
	FOREIGN KEY(level_id) REFERENCES levels(level_id),
	FOREIGN KEY(steam_id) REFERENCES players(steam_id)
) WITHOUT ROWID;

CREATE TRIGGER IF NOT EXISTS player_leaderboard_insert
	AFTER INSERT
	ON steam_leaderboard
BEGIN
	UPDATE players
	SET score_count = score_count + 1,
		evaluation_sum = evaluation_sum + NEW.evaluation
	WHERE steam_id = NEW.steam_id;
	UPDATE levels
	SET score_count = score_count + 1
	WHERE level_id = NEW.level_id;
END;

CREATE TRIGGER IF NOT EXISTS player_leaderboard_delete
	AFTER DELETE
	ON steam_leaderboard
BEGIN
	UPDATE players
	SET score_count = score_count - 1,
		evaluation_sum = evaluation_sum - OLD.evaluation
	WHERE steam_id = OLD.steam_id;
	UPDATE levels
	SET score_count = score_count + 1
	WHERE level_id = NEW.level_id;
END;

CREATE TRIGGER IF NOT EXISTS player_leaderboard_update
	AFTER UPDATE
	ON steam_leaderboard
	WHEN OLD.evaluation <> NEW.evaluation
BEGIN
	UPDATE players
	SET evaluation_sum = evaluation_sum - OLD.evaluation + NEW.evaluation
	WHERE steam_id = NEW.steam_id;
END;

CREATE TRIGGER IF NOT EXISTS player_evaluation_update
	AFTER UPDATE
	ON players
	WHEN OLD.evaluation_sum <> NEW.evaluation_sum
BEGIN
	UPDATE players
	SET evaluation = (NEW.evaluation_sum/MAX(NEW.score_count, 100))
	WHERE steam_id = NEW.steam_id;
END;

CREATE TRIGGER IF NOT EXISTS level_insert
	AFTER INSERT
	ON levels
BEGIN
	UPDATE levels
	SET evaluation = NEW.evaluation_sum/MAX(NEW.score_count, 20)
	WHERE level_id = NEW.level_id;
END;

CREATE TRIGGER IF NOT EXISTS level_update
	AFTER UPDATE
	ON levels
	WHEN OLD.evaluation_sum <> NEW.evaluation_sum
BEGIN
	UPDATE levels
	SET evaluation = NEW.evaluation_sum/MAX(NEW.score_count, 20)
	WHERE level_id = NEW.level_id;
END;

CREATE INDEX IF NOT EXISTS index_steam_leaderboard_placement ON steam_leaderboard(level_id, placement);

CREATE INDEX IF NOT EXISTS index_players_display_name ON players(cached_display_name);
CREATE INDEX IF NOT EXISTS index_players_evaluation_sum ON players(evaluation_sum);
CREATE INDEX IF NOT EXISTS index_players_evaluation ON players(evaluation);

CREATE INDEX IF NOT EXISTS index_levels_file_id ON levels(file_id);

CREATE INDEX IF NOT EXISTS index_levels_display_name ON levels(cached_display_name);
"#,
].to_vec()
}