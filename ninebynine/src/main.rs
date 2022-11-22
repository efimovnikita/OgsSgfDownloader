use std::fs::File;
use std::io::{ Write};
use clap::Parser;
use serde::{Serialize, Deserialize};
use requestty::{Question, Answer};
use std::{fs};
use std::path::{PathBuf};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::time::Duration;
use async_std::task;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::string::String;

/// Tool for download 9x9 SGF files from OGS
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Player name at OGS
    #[arg(short, long)]
    name: String,
    /// Path to save folder
    #[arg(short, long)]
    path: PathBuf
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Player {
    id: i32,
    username: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Query {
    q: String,
    players: Vec<Player>
}

#[derive(Debug, Serialize, Deserialize)]
struct GamesPage{
    count: f32,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<Game>
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    id: i32,
    width: i32,
    ended: DateTime<Utc>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    if args.path.exists() == false {
        println!("Save path not exist.");
        return Ok(());
    }

    let search_player_response: Query = reqwest::get(format!("https://online-go.com/api/v1/ui/omniSearch?q={}", args.name))
        .await?
        .json()
        .await?;

    let players = search_player_response.players;

    let mut selected_id: i32 = 0;
    let mut selected_username: String = String::new();
    if players.len() == 1 {
        selected_id = players[0].clone().id;
        selected_username = players[0].clone().username;
    }else{
        let names: Vec<String> = players.to_vec().into_iter().map(|p| p.username).collect();
        let question = Question::select("size")
            .message("Select player name")
            .choices(names)
            .build();
        let answer = requestty::prompt_one(question)?;

        if let Answer::ListItem(item) = answer {
            for player in players {
                if player.username == item.text {
                    selected_id = player.id;
                    selected_username = player.username;
                }
            }
        }
    }

    let mut games: Vec<Game> = Vec::new();

    let mut games_page: GamesPage = reqwest::get(format!("https://online-go.com/api/v1/players{}/games?page=1", selected_id))
        .await?
        .json()
        .await?;

    let pages_count = (games_page.count as f64 / 10_f64).ceil() as u64;
    let bar = ProgressBar::new(pages_count);
    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));
    for page_number in 0..pages_count as i32 {
        let secs = rand::thread_rng().gen_range(5..10);
        let duration = Duration::from_secs(secs);

        let url = String::from(format!("https://online-go.com/api/v1/players{}/games?page={}", selected_id, page_number + 1));
        bar.set_message(format!("Download {}", url));

        let response = reqwest::get(url).await;
        if response.is_err() {
            bar.set_message("Response error. Skip page...");
            task::sleep(duration).await;
            bar.inc(1);
            continue
        }

        let page = response.unwrap().json::<GamesPage>().await;
        if page.is_err() {
            bar.set_message("Deserialization error. Skip page...");
            task::sleep(duration).await;
            bar.inc(1);
            continue
        }

        games_page = page.unwrap();
        for game in games_page.results {
            if game.width == 9 {
                games.push(game);
            }
        }

        bar.inc(1);
    }
    bar.finish();

    if games.len() == 0 {
        println!("9x9 games not found.");
        return Ok(());
    }

    let mut games_grouped_by_date = Vec::new();
    for (key, group) in &games.into_iter().group_by(|game| game.ended.date_naive()) {
        games_grouped_by_date.push((key, group.collect::<Vec<Game>>()));
    }

    let mut dates = Vec::new();
    for t in &games_grouped_by_date {
        dates.push(t.0.to_string())
    }

    dates.sort();

    let question = Question::multi_select("dates")
        .message("Select dates")
        .choices(dates)
        .build();
    let answer_result = requestty::prompt_one(question);
    if answer_result.is_err() {
        println!("You must select valid dates.");
        return Ok(());
    }

    let mut selected_dates = Vec::new();
    let answer = answer_result.unwrap();
    if let Answer::ListItems(s_dates) = answer {
        for date in s_dates {
            selected_dates.push(date.text)
        }
    }

    if selected_dates.len() == 0 {
        println!("You must select some dates.");
        return Ok(());
    }

    let mut games_subset = Vec::new();
    for (date, games) in &games_grouped_by_date {
        if selected_dates.contains(&date.to_string()) {
            for game in games {
                games_subset.push(game);
            }
        }
    }

    let bar2 = ProgressBar::new((*&games_subset.len()).try_into().unwrap());
    bar2.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    let mut sgfs = Vec::new();
    for game in &games_subset {
        let url = format!("https://online-go.com/api/v1/games/{}/sgf", game.id);
        bar2.set_message(format!("Export SGF from {}", url));

        let sgf_response = reqwest::get(url).await;
        if sgf_response.is_err() {
            bar2.inc(1);
            continue
        }

        let sgf_result = sgf_response.unwrap().text().await;
        if sgf_result.is_err() {
            bar2.inc(1);
            continue
        }

        let sgf = sgf_result.unwrap();
        sgfs.push((game.id, sgf));

        bar2.inc(1);
    }

    bar2.finish();

    let dir_path = args.path.join(selected_username);
    if dir_path.exists() {
        let remove_result = fs::remove_dir_all(&dir_path);
        if remove_result.is_err() {
            println!("Error during removing directory {}", dir_path.display());
            return Ok(());
        }
    }

    let create_result = fs::create_dir(&dir_path);
    if create_result.is_err() {
        println!("Error during creating new directory {}", dir_path.display());
        return Ok(());
    }

    for (id, sgf) in &sgfs {
        let file_path = dir_path.join(format!("{}.sgf", id));
        let mut tmp_file = File::create(file_path)?;
        writeln!(tmp_file, "{}", sgf)?;
    }

    println!("{} games was downloaded and exported.", sgfs.len());

    println!("Done!");

    Ok(())
}