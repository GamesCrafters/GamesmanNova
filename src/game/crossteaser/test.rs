//! # Crossteaser Testing Module
//!
//! This module helps test crossteaser game functionality
//!
//! #### Authorship
//!

use super::*; // Import everything from the parent module

use std::collections::HashSet;

#[test]
fn test_move_into_center_from_above() {
    let mut session = Session {
        variant: None,
        length: 3,
        width: 3,
        free: 4, // Center position of a 3x3 grid is free
    };

    // Set initial orientations for all pieces, assuming uniform start for simplicity
    let initial_pieces = vec![
        Orientation {
            front: 0,
            top: 1,
            right: 2,
        }, // Top-left
        Orientation {
            front: 1,
            top: 2,
            right: 3,
        }, // Top-center
        Orientation {
            front: 2,
            top: 3,
            right: 4,
        }, // Top-right
        Orientation {
            front: 3,
            top: 4,
            right: 5,
        }, // Middle-left
        // No piece in the middle (free space)
        Orientation {
            front: 4,
            top: 5,
            right: 0,
        }, // Middle-right
        Orientation {
            front: 5,
            top: 0,
            right: 1,
        }, // Bottom-left
        Orientation {
            front: 6,
            top: 1,
            right: 2,
        }, // Bottom-center
        Orientation {
            front: 7,
            top: 2,
            right: 3,
        }, // Bottom-right
    ];

    let mut unhashed_state = UnhashedState {
        pieces: initial_pieces,
        free: session.free,
    };

    // Perform the move up, moving the top-center piece down into the center
    let new_state = session.board_down(&unhashed_state);

    // Assertions to validate the move
    assert_eq!(
        new_state.free, 1,
        "Free space should move up to the top-center."
    );
}

#[test]
fn test_move_into_center_from_left() {
    let mut session = Session {
        variant: None,
        length: 3,
        width: 3,
        free: 4, // Center position of a 3x3 grid is free
    };

    // Same initial setup as the previous test
    let initial_pieces = vec![
        Orientation {
            front: 0,
            top: 1,
            right: 2,
        }, // Top-left
        Orientation {
            front: 1,
            top: 2,
            right: 3,
        }, // Top-center
        Orientation {
            front: 2,
            top: 3,
            right: 4,
        }, // Top-right
        Orientation {
            front: 3,
            top: 4,
            right: 5,
        }, // Middle-left
        // No piece in the middle (free space)
        Orientation {
            front: 4,
            top: 5,
            right: 0,
        }, // Middle-right
        Orientation {
            front: 5,
            top: 0,
            right: 1,
        }, // Bottom-left
        Orientation {
            front: 6,
            top: 1,
            right: 2,
        }, // Bottom-center
        Orientation {
            front: 7,
            top: 2,
            right: 3,
        }, // Bottom-right
    ];

    let mut unhashed_state = UnhashedState {
        pieces: initial_pieces,
        free: session.free,
    };

    // Perform the move left, moving the middle-left piece right into the center
    let new_state = session.board_right(&unhashed_state);

    // Assertions to validate the move
    assert_eq!(
        new_state.free, 3,
        "Free space should move left to the middle-left."
    );
}

#[test]
fn test_start_end() {
    let session = Session {
        variant: None,
        length: 3,
        width: 3,
        free: 1, // Free space initially set below the center
    };
    let start_state = session.start();
    let moved_state = session.board_down(&session.unhash(start_state));
    assert_eq!(
        moved_state.free, 4,
        "Free space should move up to the center."
    );
    assert!(
        session.end(session.hash(&moved_state)),
        "This should be a valid end state."
    );
}

#[test]
fn test_transition() {
    let session: Session = Session {
        variant: None,
        length: 3,
        width: 3,
        free: 1,
    };
    let mut s: UnhashedState = UnhashedState {
        pieces: Vec::new(),
        free: 4,
    };
    for _i in 0..8 {
        s.pieces
            .push(unhash_orientation(0));
    }
    let t = session.reduce(&session.board_left(&session.board_up(&s)));

    println!("{}", session.encode(session.hash(&t)));
    let mut found: HashSet<State> = HashSet::new();
    let mut unsolved: Vec<State> = Vec::new();
    unsolved.push(session.hash(&s));
    for _ in 0..1000 {
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

#[test]
fn test_bit_array() {}
