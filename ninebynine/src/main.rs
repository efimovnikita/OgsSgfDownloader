mod structs;

use crate::structs::{
    GetGamesGroupedByDate, GetNineByNineGames, GetSgf, GetSortedDatesFromGroupedGames,
};
use async_std::task;
use chrono::NaiveDate;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use requestty::{Answer, Question};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::string::String;
use std::time::Duration;
use structs::{Args, Game, GamesPage, Player, Query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    if args.path.exists() == false {
        println!("Save path not exist.");
        return Ok(());
    }

    let search_player_result = reqwest::get(format!(
        "https://online-go.com/api/v1/ui/omniSearch?q={}",
        args.name
    ))
    .await;
    if search_player_result.is_err() {
        println!("Error response from OGS. Exit");
        return Ok(());
    }

    let query_parsing_result = search_player_result.unwrap().json::<Query>().await;
    if query_parsing_result.is_err() {
        println!("Error while parsing player info from OGS. Exit");
        return Ok(());
    }

    let search_player_response = query_parsing_result.unwrap();

    let players = search_player_response.players;

    let (selected_id, selected_username) = get_player_id_and_username(players);

    let games_page_result = reqwest::get(format!(
        "https://online-go.com/api/v1/players{}/games?page=1",
        selected_id
    ))
    .await;
    if games_page_result.is_err() {
        println!("Error response from OGS. Exit");
        return Ok(());
    }

    let unwrapped_response = games_page_result.unwrap();
    let games_page_parse_result = unwrapped_response.json::<GamesPage>().await;
    if games_page_parse_result.is_err() {
        println!("Error while parsing player games page from OGS. Exit");
        return Ok(());
    }

    let mut games_page = games_page_parse_result.unwrap();

    let pages_count = (games_page.count as f64 / 10_f64).ceil() as u64;

    let mut games: Vec<Game> = Vec::with_capacity(games_page.count as usize);

    let bar = ProgressBar::new(pages_count);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    for page_number in 0..pages_count as i32 {
        let secs = rand::thread_rng().gen_range(5..10);
        let duration = Duration::from_secs(secs);

        let url = String::from(format!(
            "https://online-go.com/api/v1/players{}/games?page={}",
            selected_id,
            page_number + 1
        ));
        bar.set_message(format!("Download {}", url));

        let response_result = reqwest::get(url).await;
        if response_result.is_err() {
            bar.set_message("Response error. Skip page...");
            task::sleep(duration).await;
            bar.inc(1);
            continue;
        }

        let page_result = response_result.unwrap().json::<GamesPage>().await;
        if page_result.is_err() {
            bar.set_message("Deserialization error. Skip page...");
            task::sleep(duration).await;
            bar.inc(1);
            continue;
        }

        games_page = page_result.unwrap();
        games.append(&mut games_page.get_nine_by_nine_games());

        bar.inc(1);
    }
    bar.finish();

    if games.len() == 0 {
        println!("9x9 games not found.");
        return Ok(());
    }

    let games_grouped_by_date: Vec<(NaiveDate, Vec<&Game>)> = games.get_games_grouped_by_date();
    let sorted_dates: Vec<String> = games_grouped_by_date.get_sorted_dates_from_grouped_games();

    let question = Question::multi_select("dates")
        .message("Select dates")
        .choices(&sorted_dates)
        .build();
    let answer_result = requestty::prompt_one(question);
    if answer_result.is_err() {
        println!("You must select valid dates.");
        return Ok(());
    }

    let mut selected_dates = Vec::with_capacity(sorted_dates.len());
    let answer = answer_result.unwrap();
    if let Answer::ListItems(s_dates) = answer {
        selected_dates = s_dates.into_iter().map(|date| date.text).collect();
    }

    if selected_dates.len() == 0 {
        println!("You must select some dates.");
        return Ok(());
    }

    let mut games_subset = Vec::with_capacity(games_page.count as usize);
    for (date, games) in games_grouped_by_date {
        if selected_dates.contains(&date.to_string()) {
            for game in games {
                games_subset.push(game);
            }
        }
    }

    let bar = ProgressBar::new((*&games_subset.len()).try_into().unwrap());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut sgfs = Vec::with_capacity(games_subset.len());
    for game in &games_subset {
        bar.set_message(format!("Export SGF for game with id \'{}\'", game.id));

        let sgf = game.get_sgf().await;
        if sgf.is_none() {
            bar.inc(1);
            continue;
        }

        sgfs.push((game.id, sgf.unwrap()));
        bar.inc(1);
    }
    bar.finish();

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

    create_sgf_files(&mut sgfs, dir_path);

    println!("{} games was downloaded and exported.", sgfs.len());

    println!("Done!");

    Ok(())
}

fn create_sgf_files(sgfs: &mut Vec<(i32, String)>, dir_path: PathBuf) {
    for (id, sgf) in sgfs {
        let file_path = dir_path.join(format!("{}.sgf", id));
        let create_file_result = File::create(&file_path);
        match create_file_result {
            Ok(mut file) => {
                let write_to_file_result = writeln!(file, "{}", sgf);
                if write_to_file_result.is_err() {
                    println!("Error when write file content: {}", file_path.display());
                    continue;
                }
            }
            Err(_) => {
                println!("Error create a file: {}", file_path.display());
                continue;
            }
        }
    }
}

fn get_player_id_and_username(players: Vec<Player>) -> (i32, String) {
    let mut selected_id: i32 = 0;
    let mut selected_username: String = String::new();
    if players.len() == 1 {
        selected_id = players[0].clone().id;
        selected_username = players[0].clone().username;
    } else {
        let names: Vec<String> = players.to_vec().into_iter().map(|p| p.username).collect();
        let question = Question::select("size")
            .message("Select player name")
            .choices(names)
            .build();
        let answer = requestty::prompt_one(question).unwrap();

        if let Answer::ListItem(item) = answer {
            for player in players {
                if player.username == item.text {
                    selected_id = player.id;
                    selected_username = player.username;
                }
            }
        }
    }
    (selected_id, selected_username)
}
