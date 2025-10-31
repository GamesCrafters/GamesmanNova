//! # Declarative Macros
//!
//! TODO

/// Declarative way of expressing extensive game state nodes.
///
/// # Example
///
/// ```ignore
/// // A medial node where it is Player 5's turn.
/// let n1 = node!(5);
///
/// // A terminal node with a 5-entry utility vector, on player 2's turn.
/// let n2 = node![2; -1, -4, 5, 0, 3];
///
/// // A terminal node with a single utility entry, on player 1's turn.
/// let n3 = node![1; 4];
/// ```
#[macro_export]
macro_rules! node {
    ($val:expr) => {
        Node::Medial($val)
    };
    ($player:expr; $($u:expr),+ $(,)?) => {
        Node::Terminal($player, vec![$($u),*])
    };
}
