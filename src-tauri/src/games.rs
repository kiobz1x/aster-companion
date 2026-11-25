use serde::Serialize;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Default)]
pub struct LolEvent {
    pub id: i64,
    pub name: String,
    pub killer: String,
    pub victim: String,
    pub streak: i64,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LolState {
    pub in_game: bool,
    pub summoner: String,
    pub champion: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub game_time_sec: f64,
    pub events: Vec<LolEvent>,
}

fn lol_http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .no_proxy()
            .danger_accept_invalid_certs(true)
            .connect_timeout(Duration::from_millis(400))
            .timeout(Duration::from_millis(900))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

#[tauri::command]
pub async fn get_lol_state() -> LolState {
    let res = match lol_http()
        .get("https://127.0.0.1:2999/liveclientdata/allgamedata")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return LolState::default(),
    };

    let Ok(body) = res.json::<serde_json::Value>().await else {
        return LolState::default();
    };

    let active_name = body["activePlayer"]["riotIdGameName"]
        .as_str()
        .or_else(|| body["activePlayer"]["summonerName"].as_str())
        .unwrap_or("")
        .to_string();

    let mut champion = String::new();
    let mut kills = 0;
    let mut deaths = 0;
    let mut assists = 0;

    if let Some(players) = body["allPlayers"].as_array() {
        for p in players {
            let p_name = p["riotIdGameName"]
                .as_str()
                .or_else(|| p["summonerName"].as_str())
                .unwrap_or("");
            if !active_name.is_empty() && p_name.eq_ignore_ascii_case(&active_name) {
                champion = p["championName"].as_str().unwrap_or("").to_string();
                kills = p["scores"]["kills"].as_i64().unwrap_or(0);
                deaths = p["scores"]["deaths"].as_i64().unwrap_or(0);
                assists = p["scores"]["assists"].as_i64().unwrap_or(0);
                break;
            }
        }
    }

    let game_time_sec = body["gameData"]["gameTime"].as_f64().unwrap_or(0.0);

    let mut events = Vec::new();
    if let Some(arr) = body["events"]["Events"].as_array() {
        for ev in arr.iter().rev().take(12) {
            let name = ev["EventName"].as_str().unwrap_or("").to_string();
            if matches!(
                name.as_str(),
                "FirstBlood" | "Multikill" | "DragonKill" | "BaronKill" | "ChampionKill" | "GameEnd"
            ) {
                events.push(LolEvent {
                    id: ev["EventID"].as_i64().unwrap_or(0),
                    name,
                    killer: ev["KillerName"].as_str().unwrap_or("").to_string(),
                    victim: ev["VictimName"].as_str().unwrap_or("").to_string(),
                    streak: ev["KillStreak"].as_i64().unwrap_or(0),
                    result: ev["Result"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        events.reverse();
    }

    LolState {
        in_game: true,
        summoner: active_name,
        champion,
        kills,
        deaths,
        assists,
        game_time_sec,
        events,
    }
}

/// Detecta jogos populares abertos pelo título da janela ativa.
#[tauri::command]
pub fn detect_known_game(window_title: String) -> Option<String> {
    let low = window_title.to_lowercase();
    let known: &[(&str, &str)] = &[
        ("league of legends", "League of Legends"),
        ("valorant", "Valorant"),
        ("counter-strike 2", "Counter-Strike 2"),
        ("minecraft", "Minecraft"),
        ("grand theft auto v", "GTA V"),
        ("cyberpunk 2077", "Cyberpunk 2077"),
        ("elden ring", "Elden Ring"),
        ("hollow knight", "Hollow Knight"),
        ("stardew valley", "Stardew Valley"),
        ("fortnite", "Fortnite"),
        ("rocket league", "Rocket League"),
        ("overwatch", "Overwatch 2"),
        ("terraria", "Terraria"),
        ("roblox", "Roblox"),
        ("dead by daylight", "Dead by Daylight"),
        ("baldur's gate 3", "Baldur's Gate 3"),
        ("palworld", "Palworld"),
        ("apex legends", "Apex Legends"),
    ];
    for (needle, pretty) in known {
        if low.contains(needle) {
            return Some((*pretty).to_string());
        }
    }
    None
}
