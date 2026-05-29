use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
}

pub fn load_scores() -> Vec<ScoreEntry> {
    let mut scores: Vec<ScoreEntry> = shared::leaderboard::load_scores();
    if scores.is_empty() {
        scores = vec![
            ScoreEntry {
                name: "BAR".to_string(),
                score: 5000,
            },
            ScoreEntry {
                name: "BUB".to_string(),
                score: 4000,
            },
            ScoreEntry {
                name: "BOB".to_string(),
                score: 3000,
            },
            ScoreEntry {
                name: "ZEN".to_string(),
                score: 2000,
            },
            ScoreEntry {
                name: "DOT".to_string(),
                score: 1000,
            },
        ];
    }
    scores
}

pub fn save_scores(scores: &[ScoreEntry]) {
    shared::leaderboard::save_scores(scores);
}

pub fn add_score(name: String, score: u32) {
    println!("Adding score for {}: {}", name, score);
    let mut scores = load_scores();
    scores.push(ScoreEntry { name, score });
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    scores.truncate(10);
    save_scores(&scores);
    println!(
        "Scores saved. Current top 3: {:?}",
        &scores.iter().take(3).collect::<Vec<_>>()
    );
}
