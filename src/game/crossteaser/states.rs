//! # Crossteaser State Handling Module
//!
//! This module helps parse the a string encoding of a crossteaser game state
//! into a more efficient binary representation, performing a series of checks
//! which partially ensure compatibility with a game variant.
//!
//! #### Authorship
//! - Max Fierro, 3/7/2023 (maxfierro@berkeley.edu)
//! - Michael Setchko Palmerlee, 4/18/2024 (michaelsp@berkeley.edu)

pub const STATE_DEFAULT: &'static str = "|0-0-0|0-X-0|0-0-0|";
pub const STATE_PATTERN: &'static str = r"^([|]([\dX]+-)+[\dX]+)+[|]";
pub const STATE_PROTOCOL: &'static str =
    "Rows are separated by |, columns are separated by -, empty space is X. \
Integers 0-23 are a piece orientation as defined by ORIENTATION_MAP";

use regex::Regex;

use crate::game::crossteaser::*;
use crate::game::error::GameError;
use crate::model::State;

pub fn parse_state(
    state: String,
    session: &Session,
) -> Result<State, GameError> {
    check_state_pattern(&state)?;
    let v: Vec<String> = parse_pieces(&state);
    for piece in &v {
        check_valid_piece(piece)?;
    }
    check_free_spaces(&v, session)?;
    check_num_pieces(&v, session)?;
    let mut rep: Vec<Orientation> = Vec::new();
    let mut empty: u64 = 0;
    for (i, piece) in v.iter().enumerate() {
        match piece.parse::<u64>() {
            Ok(n) => rep.push(unhash_orientation(n)),
            Err(_) => empty = i as u64,
        }
    }
    Ok(session.hash(&UnhashedState {
        pieces: rep,
        free: empty,
    }))
}

fn check_free_spaces(
    v: &Vec<String>,
    session: &Session,
) -> Result<(), GameError> {
    if v.iter()
        .filter(|s| *s == "X")
        .count()
        == session.free as usize
    {
        Ok(())
    } else {
        Err(GameError::StateMalformed {
            game_name: NAME,
            hint: "Invalid free space".to_owned(),
        })
    }
}

fn check_num_pieces(
    v: &Vec<String>,
    session: &Session,
) -> Result<(), GameError> {
    if v.len() == (session.width * session.length) as usize {
        Ok(())
    } else {
        Err(GameError::StateMalformed {
            game_name: NAME,
            hint: "Invalid piece count".to_owned(),
        })
    }
}

fn check_state_pattern(state: &String) -> Result<(), GameError> {
    let re = Regex::new(STATE_PATTERN).unwrap();
    if !re.is_match(&state) {
        Err(GameError::StateMalformed {
            game_name: NAME,
            hint: "Invalid pattern".to_owned(),
        })
    } else {
        Ok(())
    }
}

fn check_valid_piece(piece: &String) -> Result<(), GameError> {
    println!();
    match piece.parse::<u64>() {
        Ok(n) => {
            if n < 24 {
                Ok(())
            } else {
                Err(GameError::StateMalformed {
                    game_name: NAME,
                    hint: "Invalid pieces int".to_owned(),
                })
            }
        },
        Err(_) => {
            println!("{}", piece);
            if piece == "X" {
                Ok(())
            } else {
                Err(GameError::StateMalformed {
                    game_name: NAME,
                    hint: "Invalid pieces string".to_owned(),
                })
            }
        },
    }
}

fn parse_pieces(state: &str) -> Vec<String> {
    state
        .split(['|', '-'])
        .map(|piece| piece.to_owned())
        .collect()
}

#[cfg(test)]
mod test {
    use crate::game::crossteaser::*;
    use std::collections::HashSet;

    #[test]
    fn test_parsing() {
        let session: Session = Session {
            variant: None,
            length: 2,
            width: 3,
            free: 1,
        };
        let state: String = "|0-0-0|0-X-0|0-0-0|".to_owned();
        match parse_state(state, &session) {
            Ok(s) => println!("{}", s),
            Err(e) => println!("{}", e),
        }
    }

    #[test]
    fn test_transition() {
        let session: Session = Session {
            variant: None,
            length: 2,
            width: 3,
            free: 1,
        };
        let mut s: UnhashedState = UnhashedState {
            pieces: Vec::new(),
            free: 4,
        };
        for _i in 0..5 {
            s.pieces
                .push(unhash_orientation(0));
        }
        let mut found: HashSet<State> = HashSet::new();
        let mut unsolved: Vec<State> = Vec::new();
        unsolved.push(session.hash(&s));
        while false {
            let s: State = unsolved.pop().unwrap();
            found.insert(s);
            let f: Vec<State> = session.prograde(s);
            for state in f {
                if !found.contains(&state) {
                    unsolved.push(state);
                }
            }
            if found.len() % 100000 == 0 {
                println!("found: {}", found.len());
            }
        }
        println!("total: {}", found.len());
    }
}
