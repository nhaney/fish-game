use std::{env, fs, process};

use fish_game_replay as replay;

fn main() {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: replay-verify <replay.bin>");
            process::exit(2);
        }
    };

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("could not read {}: {}", path, e);
            process::exit(2);
        }
    };

    let recorded = match replay::Replay::decode(&bytes) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("could not decode replay: {}", e);
            process::exit(2);
        }
    };

    let result = replay::verify(&recorded);
    println!("recorded target : {}", recorded.target_triple);
    println!("expected hash   : {:#018x}", recorded.final_hash);
    println!("actual hash     : {:#018x}", result.actual_hash);
    println!("expected score  : {}", recorded.final_score);
    println!("actual score    : {}", result.actual_score);

    if result.hash_matches && result.score_matches {
        println!("PASS");
        process::exit(0);
    }
    println!(
        "FAIL (hash_matches={}, score_matches={})",
        result.hash_matches, result.score_matches
    );
    process::exit(1);
}
