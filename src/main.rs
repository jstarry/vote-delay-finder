use chrono::{DateTime, FixedOffset};
use regex::Regex;
use rev_lines::RevLines;
use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        eprintln!("Must provide a file path");
    }
    let file_name = &args[1];
    println!("Searching for long vote delays in {}", file_name);
    let file = File::open(file_name).unwrap();
    let rev_lines = RevLines::new(BufReader::new(file)).unwrap();
    let vote_bank_regex = Regex::new(r"\[(.*) INFO .* vote bank: Some\(\((\d*)").unwrap();
    let voting_regex = Regex::new(r"\[(.*) INFO .* voting: (\d*)").unwrap();

    let mut bank_vote_times = HashMap::<u64, DateTime<FixedOffset>>::new();
    for line in rev_lines {
        if let Some(captures) = vote_bank_regex.captures(&line) {
            let vote_timestamp = DateTime::parse_from_rfc3339(&captures[1]).unwrap();
            let vote_slot = u64::from_str(&captures[2]).unwrap();
            bank_vote_times.insert(vote_slot, vote_timestamp);
        } else if let Some(captures) = voting_regex.captures(&line) {
            let vote_timestamp = DateTime::parse_from_rfc3339(&captures[1]).unwrap();
            let vote_slot = u64::from_str(&captures[2]).unwrap();
            if vote_slot % 1000 == 0 {
                println!(".. processing slot {}", vote_slot);
            }
            if let Some(vote_bank_timestamp) = bank_vote_times.remove(&vote_slot) {
                let vote_epoch_ms = vote_timestamp.timestamp_millis();
                let vote_bank_epoch_ms = vote_bank_timestamp.timestamp_millis();
                let vote_elapsed_ms = vote_bank_epoch_ms.saturating_sub(vote_epoch_ms);
                if vote_elapsed_ms > 1000 {
                    println!("vote: {}, elapsed: {:>5.2}s", vote_slot, vote_elapsed_ms as f64 / 1000.0);
                }
            }
        }

    }

}
