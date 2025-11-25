//! board decsription

use crate::synchronization::{interface::Mutex, NullLock};

pub mod interface {

    /// Board information
    pub trait Info {
        fn board_name(&self) -> &'static str;
    }

    pub trait All: Info {}
}

/// A placeholder.
struct NullBoard;

impl interface::Info for NullBoard {
    fn board_name(&self) -> &'static str {
        "Null Board"
    }
}

impl interface::All for NullBoard {}

static NULL_BOARD: NullBoard = NullBoard {};

static CURR_BOARD: NullLock<&'static (dyn interface::All + Sync)> = NullLock::new(&NULL_BOARD);

/// Register a new board.
pub fn register_board(new_board: &'static (dyn interface::All + Sync)) {
    CURR_BOARD.lock(|brd| *brd = new_board);
}

/// Return a reference to the console.
pub fn board() -> &'static dyn interface::All {
    CURR_BOARD.lock(|brd| *brd)
}
