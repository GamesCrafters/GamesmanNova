//! # Strong Solving Module
//!
//! This module implements strong solvers for all applicable types of games
//! through blanket implementations. Solvers that take advantage of different
//! game structures are under modules whose names describe what that structure
//! is. For example, the blanket implementation for `strong::acyclic::Solver`
//! should provide a strong solver for all `Acyclic` games through a blanket
//! implementation.
//!
//! #### Authorship
//!
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)

/// # Strong Acyclic Solving Module
///
/// This module implements strong acyclic solvers for all applicable types of
/// games through blanket implementations of the `acyclic::Solver` trait,
/// optimizing for specific game characteristics wherever possible.
///
/// #### Authorship
///
/// - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)
pub mod acyclic {
    use crate::database::record::Record;
    use crate::database::{bpdb::BPDatabase, Database};
    use crate::games::{Acyclic, DynamicAutomaton, StaticAutomaton};
    use crate::interfaces::terminal::cli::IOMode;
    use crate::models::{PlayerCount, State};
    use crate::solvers::MAX_TRANSITIONS;

    /* SOLVER INTERFACES */

    /// Provides behavior for strongly solving games whose states do not repeat
    /// during play. A strong solution inherently visits all positions in the
    /// game, determines the best possible strategy for the player whose turn it
    /// is at that position, and associates the value achievable through that
    /// strategy to the position. This interface allows for an arbitrary number
    /// of state transitions being possible from all game states by using a
    /// dynamically-sized data structure at the cost of performance.
    ///
    /// This interface only provides non-pure behavior for generating this
    /// association (i.e., a solution set) by writing to a file. See the
    /// arguments of `solve` for specifics.
    pub trait DynamicSolver<const N: PlayerCount, S>
    where
        Self: Acyclic<N> + DynamicAutomaton<S>,
    {
        /// Has the side effect of writing a strong solution to the underlying
        /// game by exploring using `from` as a starting state. For specifics
        /// on when files are written, see `cli::IOMode`. This function assumes
        /// that `from` is a valid state encoding of the underlying game.
        fn solve(&self, mode: IOMode, from: S);
    }

    /// Provides behavior for strongly solving games whose states do not repeat
    /// during play. A strong solution inherently visits all positions in the
    /// game, determines the best possible strategy for the player whose turn it
    /// is at that position, and associates the value achievable through that
    /// strategy to the position. This interface specifies that the traversal
    /// of the game must be possible via stack-allocated data structures by
    /// requiring the implementation of `StaticAutomaton` with a preset number
    /// of maximum state transitions `solvers::MAX_TRANSITIONS`.
    ///
    /// This interface only provides non-pure behavior for generating this
    /// association (i.e., a solution set) by writing to a file. See the
    /// arguments of `solve` for specifics.
    pub trait StaticSolver<const N: PlayerCount, const F: usize, S>
    where
        Self: Acyclic<N> + StaticAutomaton<S, F>,
    {
        /// Has the side effect of writing a strong solution to the underlying
        /// game by exploring using `from` as a starting state. For specifics
        /// on when files are written, see `cli::IOMode`. This function assumes
        /// that `from` is a valid state encoding of the underlying game.
        fn solve(&self, mode: IOMode, from: S);
    }

    /* BLANKET IMPLEMENTATIONS */

    impl<const N: PlayerCount, G> DynamicSolver<N, State> for G
    where
        G: Acyclic<N> + DynamicAutomaton<State>,
    {
        fn solve(&self, mode: IOMode, from: State) {
            let mut db = BPDatabase::new(self.id(), mode);
            dfs_heap_reverse_induction(&mut db, from, self);
        }
    }

    impl<const N: PlayerCount, G> StaticSolver<N, MAX_TRANSITIONS, State> for G
    where
        G: Acyclic<N> + StaticAutomaton<State, MAX_TRANSITIONS>,
    {
        fn solve(&self, mode: IOMode, from: State) {
            let mut db = BPDatabase::new(self.id(), mode);
            dfs_stack_reverse_induction(&mut db, from, self);
        }
    }

    /* SOLVING ALGORITHMS */

    fn dfs_heap_reverse_induction<
        const N: PlayerCount,
        G: DynamicSolver<N, State>,
    >(
        db: &mut BPDatabase<N>,
        from: State,
        game: &G,
    ) {
        let mut stack = Vec::new();
        stack.push(from);
        while let Some(curr) = stack.pop() {
            let children = game.transition(curr);
            if let None = db.get(curr) {
                db.put(curr, Record::default());
                if children.is_empty() {
                    let record = Record::default()
                        .with_utility(game.utility(curr))
                        .with_remoteness(0);
                    db.put(curr, record)
                } else {
                    stack.push(curr);
                    stack.extend(children.iter());
                }
            } else {
                db.put(
                    curr,
                    children
                        .iter()
                        .map(|&x| db.get(x).unwrap())
                        .max_by(|r1, r2| r1.cmp(&r2, game.turn(curr)))
                        .unwrap(),
                );
            }
        }
    }

    fn dfs_stack_reverse_induction<
        const N: PlayerCount,
        G: StaticSolver<N, MAX_TRANSITIONS, State>,
    >(
        db: &mut BPDatabase<N>,
        from: State,
        game: &G,
    ) {
        let mut stack = Vec::new();
        stack.push(from);
        while let Some(curr) = stack.pop() {
            let children = game.transition(curr);
            if let None = db.get(curr) {
                db.put(curr, Record::default());
                if children
                    .iter()
                    .all(|x| x.is_none())
                {
                    let record = Record::default()
                        .with_utility(game.utility(curr))
                        .with_remoteness(0);
                    db.put(curr, record)
                } else {
                    stack.push(curr);
                    stack.extend(children.iter().filter_map(|&x| x));
                }
            } else {
                db.put(
                    curr,
                    children
                        .iter()
                        .filter_map(|&x| x)
                        .map(|x| db.get(x).unwrap())
                        .max_by(|r1, r2| r1.cmp(&r2, game.turn(curr)))
                        .unwrap(),
                );
            }
        }
    }
}

/// # Strong Cyclic Solving Module
///
/// This module implements strong cyclic solvers for all applicable types of
/// games through blanket implementations of the `acyclic::Solver` trait,
/// optimizing for specific game characteristics wherever possible.
///
/// #### Authorship
///
/// - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)
pub mod cyclic {}
