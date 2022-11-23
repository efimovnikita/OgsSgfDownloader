use clap::Parser;
use serde::{Serialize, Deserialize};
use std::path::{PathBuf};
use chrono::{DateTime, Utc};

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

pub trait IsNineByNine {
    fn is_nine_by_nine(&self) -> bool;
}

impl IsNineByNine for Game {
    fn is_nine_by_nine(&self) -> bool {
        self.width == 9
    }
}