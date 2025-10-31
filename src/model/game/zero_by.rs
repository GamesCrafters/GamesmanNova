//! Zero-By Game Models
//!
//! TODO

use crate::model::database::Schema;
use crate::model::game::PlayerCount;
use crate::model::game::State;

/* TYPES */

pub type Elements = u64;

/* GAME METADATA */

pub const NAME: &str = "zero-by";
pub const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
pub const ABOUT: &str = "Many players take turns removing a number of elements \
from a set of arbitrary size. The game variant determines how many players are \
in the game, how many elements are in the set to begin with, and the options \
players have in the amount of elements to remove during their turn. The player \
who is left with 0 elements in their turn loses. A player cannot remove more \
elements than currently available in the set.";

/* VARIANT ENCODING */

pub const VARIANT_DEFAULT: &str = "2-10-1-2";
pub const VARIANT_PATTERN: &str = r"^[1-9]\d*(?:-[1-9]\d*)+$";
pub const VARIANT_PROTOCOL: &str = "The variant should be a dash-separated \
group of three or more positive integers. For example, '4-232-23-6-3-6' is \
valid but '598', '-23-1-5', and 'fifteen-2-5' are not. The first integer \
represents the number of players in the game. The second integer represents \
the number of elements in the set. The rest are choices that the players have \
when they need to remove a number of pieces on their turn. Note that the \
numbers can be repeated, but if you repeat the first number it will be a win \
for the player with the first turn in 1 move. If you repeat any of the rest \
of the numbers, the only consequence will be a slight decrease in performance.";

/* STATE ENCODING */

pub const STATE_DEFAULT: &str = "10-0";
pub const STATE_PATTERN: &str = r"^\d+-\d+$";
pub const STATE_PROTOCOL: &str = "Two dash-separated positive integers. The \
first integer indicates the amount of elements left to remove from the set, \
and the second indicates whose turn it is to remove an element. The first \
integer must be less than or equal to the number of initial elements specified \
by the game variant. Likewise, the second integer must be strictly less than \
the number of players in the game.";

/* GAME REPRESENTATION */

pub struct Session {
    pub start_elems: Elements,
    pub start_state: State,
    pub player_bits: usize,
    pub players: PlayerCount,
    pub schema: Schema,
    pub name: String,
    pub by: Vec<Elements>,
}
