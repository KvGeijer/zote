#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct CodeLoc {
    index: u32,
    line: u16,
    col: u16,
}

impl CodeLoc {
    pub fn new(index: usize, line: usize, col: usize) -> Self {
        Self {
            index: index as u32,
            line: line as u16,
            col: col as u16,
        }
    }

    // Use getters in case I want to bit pack in another way in the future
    pub fn index(&self) -> usize {
        self.index as usize
    }

    pub fn line(&self) -> usize {
        self.line as usize
    }

    pub fn col(&self) -> usize {
        self.col as usize
    }

    pub fn adv_col(&mut self, nbr_chars: usize, nbr_ind: usize) {
        self.index += nbr_ind as u32;
        self.col += nbr_chars as u16;
    }

    pub fn adv_line(&mut self) {
        self.index += 1;
        self.line += 1;
        self.col = 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRange {
    start: CodeLoc,
    end: CodeLoc,
}

impl CodeRange {
    pub fn from_locs(start: CodeLoc, end: CodeLoc) -> Self {
        Self { start, end }
    }

    pub fn from_ints(
        sindex: u32,
        sline: u16,
        scol: u16,
        eindex: u32,
        eline: u16,
        ecol: u16,
    ) -> Self {
        Self {
            start: CodeLoc::new(sindex as usize, sline as usize, scol as usize),
            end: CodeLoc::new(eindex as usize, eline as usize, ecol as usize),
        }
    }

    pub fn sl(&self) -> u16 {
        self.start.line
    }
    pub fn sc(&self) -> u16 {
        self.start.col
    }
    pub fn el(&self) -> u16 {
        self.end.line
    }
    pub fn ec(&self) -> u16 {
        self.end.col
    }
}
