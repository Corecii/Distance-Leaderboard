# insert data

INSERT INTO players (steam_id) VALUES ('abc')
	ON CONFLICT(steam_id) DO NOTHING;

INSERT INTO levels (level_id) VALUES ('123')
	ON CONFLICT(level_id) DO NOTHING;

INSERT INTO steam_leaderboard (level_id, steam_id, time) VALUES ('123', 'abc', 0)
	ON CONFLICT(level_id, steam_id) DO UPDATE SET time = excluded.time;

# update placement, score on whole level's leaderboard

DROP TABLE IF EXISTS temp.row_nums;
CREATE TEMPORARY TABLE row_nums as 
	SELECT
		(ROW_NUMBER() OVER (ORDER BY time ASC)) row_num,
		steam_id
	FROM steam_leaderboard
	WHERE level_id = 'Cataclysm_1_stable';
UPDATE steam_leaderboard
SET
	placement = (SELECT row_num FROM temp.row_nums WHERE steam_id = steam_leaderboard.steam_id),
	score = ((SELECT score_count FROM levels WHERE level_id = steam_leaderboard.level_id) - placement)
WHERE
	level_id = 'Cataclysm_1_stable';

# update score_sum on all levels

UPDATE levels
SET
	score_sum = (
		SELECT SUM(score_sum)
		FROM (SELECT (SELECT score_sum FROM players WHERE steam_id = steam_leaderboard.steam_id) AS score_sum FROM steam_leaderboard WHERE level_id = levels.level_id)
	)

# update score_sum on level

UPDATE levels
SET
	score_sum = (SELECT SUM(score_sum) FROM players WHERE steam_id = (SELECT steam_id FROM steam_leaderboard WHERE level_id = '123'))
WHERE
	level_id = '123';

# update score_sum on every level a player has played

UPDATE levels
SET
	score_sum = (SELECT SUM(score_sum) FROM players WHERE steam_id = (SELECT steam_id FROM steam_leaderboard WHERE level_id = '123'))
WHERE
	level_id = (SELECT level_id from steam_leaderboard WHERE steam_id = 'abc');