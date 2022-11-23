use clap::Parser;
use serde::{Serialize, Deserialize};
use std::path::{PathBuf};
use chrono::{DateTime, Utc, NaiveDate};
use itertools::Itertools;

/// Tool for download 9x9 SGF files from OGS
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Player name at OGS
    #[arg(short, long)]
    pub name: String,
    /// Path to save folder
    #[arg(short, long)]
    pub path: PathBuf
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: i32,
    pub username: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    q: String,
    pub players: Vec<Player>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GamesPage{
    pub count: f32,
    next: Option<String>,
    previous: Option<String>,
    pub results: Vec<Game>
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Game {
    pub id: i32,
    width: i32,
    pub ended: DateTime<Utc>
}

pub trait GetNineByNineGames {
    fn get_nine_by_nine_games(&self) -> Vec<Game>;
}

impl GetNineByNineGames for GamesPage {
    fn get_nine_by_nine_games(&self) -> Vec<Game> {
        self
            .results
            .iter()
            .copied()
            .filter(|game| game.is_nine_by_nine())
            .collect()
    }
}

trait IsNineByNine {
    fn is_nine_by_nine(&self) -> bool;
}

impl IsNineByNine for Game {
    fn is_nine_by_nine(&self) -> bool {
        self.width == 9
    }
}

pub trait GetGamesGroupedByDate {
    fn get_games_grouped_by_date(&self) -> Vec<(NaiveDate, Vec<&Game>)>;
}

impl GetGamesGroupedByDate for Vec<Game> {
    fn get_games_grouped_by_date(&self) -> Vec<(NaiveDate, Vec<&Game>)> {
        self
            .iter()
            .group_by(|game| game.ended.date_naive())
            .into_iter()
            .map(|(key, group)| (key, group.collect()))
            .collect()
    }
}

pub trait GetSortedDatesFromGroupedGames {
    fn get_sorted_dates_from_grouped_games(&self) -> Vec<String>;
}

impl GetSortedDatesFromGroupedGames for Vec<(NaiveDate, Vec<&Game>)> {
    fn get_sorted_dates_from_grouped_games(&self) -> Vec<String> {
        let mut dates: Vec<String> = self
            .iter()
            .map(|(d, _v)| d.to_string())
            .collect();
        dates.sort();
        dates
    }
}