use core::fmt;

use crate::code_loc::CodeLoc;

const NBR_COL_BITS: usize = 12;
const NBR_ROW_BITS: usize = 32 - NBR_COL_BITS;

const ROW_START_SHIFT: usize = 2 * NBR_COL_BITS + NBR_ROW_BITS;
const ROW_END_SHIFT: usize = NBR_COL_BITS;
const COL_START_SHIFT: usize = NBR_COL_BITS + NBR_ROW_BITS;
const COL_END_SHIFT: usize = 0;

// To keep track of the location of nodes in the AST
#[derive(Debug, Clone, Copy)]
pub struct AstLoc {
    // (start_row, start_col, end_row, end_col)
    // Inspired by the Beaver parser generator
    loc: usize,
}

impl AstLoc {
    pub fn row_start(&self) -> usize {
        let mask = ((1 << NBR_ROW_BITS) - 1) << ROW_START_SHIFT;
        (self.loc & mask) >> ROW_START_SHIFT
    }
    pub fn row_end(&self) -> usize {
        let mask = ((1 << NBR_ROW_BITS) - 1) << ROW_END_SHIFT;
        (self.loc & mask) >> ROW_END_SHIFT
    }
    pub fn col_start(&self) -> usize {
        let mask = ((1 << NBR_COL_BITS) - 1) << COL_START_SHIFT;
        (self.loc & mask) >> COL_START_SHIFT
    }
    pub fn col_end(&self) -> usize {
        let mask = ((1 << NBR_COL_BITS) - 1) << COL_END_SHIFT;
        (self.loc & mask) >> COL_END_SHIFT
    }

    pub fn new(row_start: usize, row_end: usize, col_start: usize, col_end: usize) -> AstLoc {
        assert!(row_end >= row_start);
        if row_start.max(row_end) >= (1 << NBR_ROW_BITS) {
            panic!("Too many rows! (upper limit of {})", 1 << NBR_ROW_BITS);
        }
        if col_start.max(col_end) >= (1 << NBR_COL_BITS) {
            panic!("Too many cols! (upper limit of {})", 1 << NBR_COL_BITS);
        }

        Self {
            loc: (row_start << ROW_START_SHIFT)
                | (row_end << ROW_END_SHIFT)
                | (col_start << COL_START_SHIFT)
                | (col_end << COL_END_SHIFT),
        }
    }
}

impl fmt::Display for AstLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.row_start(),
            self.col_start(),
            self.row_end(),
            self.col_end()
        )
    }
}
