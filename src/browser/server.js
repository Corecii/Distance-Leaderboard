const SQL = require('sql-template-strings')
const express = require('express');
const sqlite = require('sqlite');
const delay = require('delay');
const fs = require('fs-extra');
const path = require('path');

//

const sql_select_level_details = "SELECT * FROM levels WHERE level_id = ?1"

const sql_select_player_details = "SELECT * FROM players WHERE steam_id = ?1"

//

const sql_select_players = "SELECT (row_number() OVER(ORDER BY evaluation DESC)) AS placement, * FROM players"
const sql_select_players_placement =
    "WITH placement_players AS (SELECT (row_number() OVER(ORDER BY evaluation DESC)) AS placement, * FROM players)"
    + "SELECT * FROM placement_players WHERE steam_id = ?1"
const sql_select_players_near =
    "WITH placement_players AS (SELECT (row_number() OVER(ORDER BY evaluation DESC)) AS placement, * FROM players)"
    + " SELECT * FROM placement_players WHERE placement >= ?1 AND placement <= ?2"

const sql_select_levels = "SELECT evaluation AS difficulty, * FROM levels ORDER BY difficulty DESC"
const sql_select_levels_near =
    "WITH"
    + " levels_difficulty AS (SELECT evaluation AS difficulty,  * FROM levels ORDER BY difficulty DESC),"
    + " levels_difficulty_row AS (SELECT ROW_NUMBER() OVER(ORDER BY difficulty DESC) AS placement, * FROM levels_difficulty)"
    + "SELECT * FROM levels_difficulty_row WHERE placement >= ?1 AND placement <= ?2"

const sql_select_level_lb = "SELECT *, (SELECT cached_display_name FROM players WHERE steam_id = steam_leaderboard.steam_id) as cached_display_name FROM steam_leaderboard WHERE level_id = ?1 ORDER BY placement ASC"
const sql_select_level_lb_placement = "SELECT *, (SELECT cached_display_name FROM players WHERE steam_id = steam_leaderboard.steam_id) as cached_display_name FROM steam_leaderboard WHERE level_id = ?1 AND steam_id = ?2"
const sql_select_level_lb_near =
    "SELECT *, (SELECT cached_display_name FROM players WHERE steam_id = steam_leaderboard.steam_id) as cached_display_name FROM steam_leaderboard WHERE level_id = ?1 AND placement >= ?2 AND placement <= ?3 ORDER BY placement ASC"

const sql_select_player_lb_near =
    "WITH levels_player AS (SELECT ROW_NUMBER() OVER(ORDER BY evaluation DESC) AS row_num, *, (SELECT cached_display_name FROM levels WHERE level_id = steam_leaderboard.level_id) AS cached_display_name FROM steam_leaderboard WHERE steam_id = ?1 ORDER BY evaluation DESC)"
    + " SELECT * FROM levels_player WHERE row_num >= ?2 AND row_num <= ?3"

//

async function waitExists(filePath) {
    while (!(await fs.pathExists(filePath))) {
        await delay(1000);
    }
}

function zeroFill( number, width )
{
  width -= number.toString().length;
  if ( width > 0 )
  {
    return new Array( width + (/\./.test( number ) ? 2 : 1) ).join( '0' ) + number;
  }
  return number + ""; // always return a string
}

//

let dbPath = "/root/output/distance_leaderboard.db";

let dbPromise = waitExists(dbPath).then(() => sqlite.open(dbPath, sqlite.OPEN_READONLY));

//

const app = express();
const port = 80;

app.set('view engine', 'pug');
app.set('views', path.join(__dirname, '/views'));
app.use(express.static(path.join(__dirname, '/public')));

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
            let mil = entry.score%1000;
            let sec = Math.floor(entry.score/1000)%60;
            let min = Math.floor(entry.score/60000);
            entry.score = `${min}:${zeroFill(sec, 2)}.${zeroFill(mil, 3)}`;
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
            let mil = entry.score%1000;
            let sec = Math.floor(entry.score/1000)%60;
            let min = Math.floor(entry.score/60000);
            entry.score = `${min}:${zeroFill(sec, 2)}.${zeroFill(mil, 3)}`;
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