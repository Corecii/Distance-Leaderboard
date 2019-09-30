const SQL = require('sql-template-strings')
const express = require('express');
const sqlite = require('sqlite');

//

const sql_select_level_details = "SELECT * FROM levels WHERE level_id = ?1"

const sql_select_player_details = "SELECT * FROM players WHERE steam_id = ?1"

//

const sql_select_players = "SELECT (row_number() OVER(ORDER BY score_sum DESC)) AS placement, * FROM players"
const sql_select_players_placement =
    "WITH placement_players AS (SELECT (row_number() OVER(ORDER BY score_sum DESC)) AS placement, * FROM players)"
    + "SELECT * FROM placement_players WHERE steam_id = ?1"
const sql_select_players_near =
    "WITH placement_players AS (SELECT (row_number() OVER(ORDER BY score_sum DESC)) AS placement, * FROM players)"
    + " SELECT * FROM placement_players WHERE placement >= ?1 AND placement <= ?2"

const sql_select_levels = "SELECT (score_sum/score_count)*min(score_count, 30) AS difficulty, * FROM levels ORDER BY difficulty DESC"
const sql_select_levels_near =
    "WITH"
    + " levels_difficulty AS (SELECT (score_sum/score_count)*min(score_count, 30) AS difficulty,  * FROM levels ORDER BY difficulty DESC),"
    + " levels_difficulty_row AS (SELECT ROW_NUMBER() OVER(ORDER BY difficulty DESC) AS placement, * FROM levels_difficulty)"
    + "SELECT * FROM levels_difficulty_row WHERE placement >= ?1 AND placement <= ?2"

const sql_select_level_lb = "SELECT *, (SELECT cached_display_name FROM players WHERE steam_id = steam_leaderboard.steam_id) as cached_display_name FROM steam_leaderboard WHERE level_id = ?1 ORDER BY placement ASC"
const sql_select_level_lb_placement = "SELECT *, (SELECT cached_display_name FROM players WHERE steam_id = steam_leaderboard.steam_id) as cached_display_name FROM steam_leaderboard WHERE level_id = ?1 AND steam_id = ?2"
const sql_select_level_lb_near =
    "SELECT *, (SELECT cached_display_name FROM players WHERE steam_id = steam_leaderboard.steam_id) as cached_display_name FROM steam_leaderboard WHERE level_id = ?1 AND placement >= ?2 AND placement <= ?3 ORDER BY placement ASC"

const sql_select_player_lb_near =
    "WITH levels_player AS (SELECT ROW_NUMBER() OVER(ORDER BY score DESC) AS row_num, *, (SELECT cached_display_name FROM levels WHERE level_id = steam_leaderboard.level_id) AS cached_display_name FROM steam_leaderboard WHERE steam_id = ?1 ORDER BY score DESC)"
    + " SELECT * FROM levels_player WHERE row_num >= ?2 AND row_num <= ?3"

//

let dbPromise = sqlite.open("./test.db", sqlite.OPEN_READONLY);

//

const app = express();
const port = 3000;

app.set('view engine', 'pug');
app.use(express.static('/public'));

async function respondPlayers(req, res) {
    try {
        const db = await dbPromise;
        let entries = await db.all(sql_select_players_near, [req.params.start, req.params.start + req.params.count]);
        res.render('players', {
            title: 'Players',
            browserTitle: 'Players',
            entries: entries,
            prevPage: `/players/${req.params.start - req.params.count - 1}`,
            nextPage: `/players/${req.params.start + req.params.count + 1}`
        })
    } catch(err) {
        console.error(err);
        res.sendStatus(500);
    }
}

app.get('/players', async (req, res) => {
    req.params.start = 0;
    req.params.count = 30;
    await respondPlayers(req, res);
});

app.get('/players/:start', async (req, res) => {
    req.params.start = Number(req.params.start);
    req.params.count = 30;
    if (req.params.start != req.params.start || req.params.start%1 != 0) {
        res.sendStatus(400);
    } else {
        await respondPlayers(req, res);
    }
});

async function respondLevels(req, res) {
    try {
        const db = await dbPromise;
        let entries = await db.all(sql_select_levels_near, [req.params.start, req.params.start + req.params.count]);
        res.render('levels', {
            title: 'Levels',
            browserTitle: 'Levels',
            entries: entries,
            prevPage: `/levels/${req.params.start - req.params.count - 1}`,
            nextPage: `/levels/${req.params.start + req.params.count + 1}`
        });
    } catch(err) {
        console.error(err);
        res.sendStatus(500);
    }
}

app.get('/levels', async (req, res) => {
    req.params.start = 0;
    req.params.count = 30;
    await respondLevels(req, res);
});

app.get('/levels/:start', async (req, res) => {
    req.params.start = Number(req.params.start);
    req.params.count = 30;
    if (req.params.start != req.params.start || req.params.start%1 != 0) {
        res.sendStatus(400);
    } else {
        await respondLevels(req, res);
    }
})

async function respondLevel(req, res) {
    try {
        const db = await dbPromise;
        let details = await db.get(sql_select_level_details, [req.params.levelId]);
        if (!details) {
            res.sendStatus(404);
            return;
        }
        let entries = await db.all(sql_select_level_lb_near, [req.params.levelId, req.params.start, req.params.start + req.params.count]);
        for (let entry of entries) {
            let mil = entry.time%1000;
            let sec = Math.floor(entry.time/1000)%60;
            let min = Math.floor(entry.time/60000);
            entry.time = `${min}:${sec}.${mil}`;
        }
        res.render('level', {
            title: `Level: ${details.cached_display_name}`,
            browserTitle: `Level: ${details.cached_display_name}`,
            entries: entries,
            prevPage: `/level/${req.params.levelId}/${req.params.start - req.params.count - 1}`,
            nextPage: `/level/${req.params.levelId}/${req.params.start + req.params.count + 1}`
        });
    } catch(err) {
        console.error(err);
        res.sendStatus(500);
    }
}

app.get('/level/:levelId', async (req, res) => {
    req.params.start = 0;
    req.params.count = 30;
    await respondLevel(req, res);
});

app.get('/level/:levelId/:start', async (req, res) => {
    req.params.start = Number(req.params.start);
    req.params.count = 30;
    if (req.params.start != req.params.start || req.params.start%1 != 0) {
        res.sendStatus(400);
    } else {
        await respondLevel(req, res);
    }
});

async function respondPlayer(req, res) {
    try {
        const db = await dbPromise;
        let details = await db.get(sql_select_player_details, [req.params.playerId]);
        if (!details) {
            res.sendStatus(404);
            return;
        }
        let entries = await db.all(sql_select_player_lb_near, [req.params.playerId, req.params.start, req.params.start + req.params.count]);
        for (let entry of entries) {
            let mil = entry.time%1000;
            let sec = Math.floor(entry.time/1000);
            let min = Math.floor(entry.time/60000);
            entry.time = `${min}:${sec}.${mil}`;
        }
        res.render('player', {
            title: `Player: ${details.cached_display_name}`,
            browserTitle: `Player: ${details.cached_display_name}`,
            entries: entries,
            prevPage: `/player/${req.params.playerId}/${req.params.start - req.params.count - 1}`,
            nextPage: `/player/${req.params.playerId}/${req.params.start + req.params.count + 1}`
        });
    } catch(err) {
        console.error(err);
        res.sendStatus(500);
    }
}

app.get('/player/:playerId', async (req, res) => {
    req.params.start = 0;
    req.params.count = 30;
    await respondPlayer(req, res);
});

app.get('/player/:playerId/:start', async (req, res) => {
    req.params.start = Number(req.params.start);
    req.params.count = 30;
    if (req.params.start != req.params.start || req.params.start%1 != 0) {
        res.sendStatus(400);
    } else {
        await respondPlayer(req, res);
    }
});

app.get('/', async (req, res) => {
    req.params.start = 0;
    req.params.count = 30;
    await respondPlayers(req, res);
});

app.listen(port, () => console.log(`Listening on port ${port}!`));

app.use(function (req, res, next) {
    res.sendStatus(404);
});