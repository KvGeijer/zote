use super::{Chunk, CompRes, CompRetRes};

/// Keeps track of point with exceptional flow points
///
/// For example, loops and functions have exception-like behaviour with `break`,
/// `continue` and `return`, which needs more than local information to compile.
/// This keeps track of those currently unresolved points.
pub struct FlowPoints {
    points: Vec<FlowPoint>,
}

enum FlowPoint {
    LoopEntry { pc: usize },
    LoopExit { reserved: usize },
    BreakExit { reserved: usize },
}

impl FlowPoints {
    pub fn new() -> Self {
        Self { points: vec![] }
    }
    
    pub fn push_loop_entry(&mut self, pc: usize) {
        self.points.push(FlowPoint::LoopEntry { pc })
    }

    pub fn push_loop_exit(&mut self, reserved: usize) {
        self.points.push(FlowPoint::LoopExit { reserved })
    }

    pub fn push_break_exit(&mut self, reserved: usize) -> CompRes {
        // TODO: Better error check with functions
        if !self.points.is_empty() {
            self.points.push(FlowPoint::BreakExit { reserved });
            Ok(())
        } else {
            Err(format!("Break encountered outside loop"))
        }
    }

    /// Closes a loop at the top of the chunk, by updating reserved labels
    pub fn close_loop(&mut self, chunk: &mut Chunk) -> CompRes {
        loop {
            let Some(flow_point) = self.points.pop()  else { 
                return Err(format!("Could not close loop without an opening."))
            };

            match flow_point {
                FlowPoint::LoopEntry { pc:_ } => panic!("Loop entry encountered when closing loop"),
                FlowPoint::LoopExit { reserved } => {
                    chunk.patch_reserved_jump(reserved);
                    break
                },
                FlowPoint::BreakExit { reserved } => {
                    chunk.patch_reserved_jump(reserved)
                },
            }
        }

        // Now that loop labels are updated, and loop closed, remove its entry
        if let Some(FlowPoint::LoopEntry{pc:_}) = self.points.pop() {
             Ok(())
        } else {
            panic!("Loop exit should have matching entry")
        }
    }

    pub fn get_loop_entry(&mut self) -> CompRetRes<usize> {
        for flow_point in self.points.iter().rev() {
            match flow_point {
                FlowPoint::LoopEntry { pc } => return Ok(*pc),
                FlowPoint::LoopExit { reserved:_ } | FlowPoint::BreakExit { reserved:_ } => (),
            }
        }
        Err(format!("Cannot use continue outside of loops"))
    }
}
