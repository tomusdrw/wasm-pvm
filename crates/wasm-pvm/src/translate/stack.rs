const FIRST_STACK_REG: u8 = 2;
const LAST_STACK_REG: u8 = 6;
const STACK_REG_COUNT: usize = (LAST_STACK_REG - FIRST_STACK_REG + 1) as usize;

#[derive(Debug)]
pub struct StackMachine {
    depth: usize,
}

impl StackMachine {
    pub const fn new() -> Self {
        Self { depth: 0 }
    }

    pub fn push(&mut self) -> u8 {
        let reg = self.top_reg();
        self.depth += 1;
        reg
    }

    pub fn pop(&mut self) -> u8 {
        assert!(self.depth > 0, "Stack underflow");
        self.depth -= 1;
        self.top_reg()
    }

    pub fn peek(&self, offset: usize) -> u8 {
        assert!(offset < self.depth, "Stack peek out of bounds");
        let idx = self.depth - 1 - offset;
        Self::reg_for_depth(idx)
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    fn top_reg(&self) -> u8 {
        Self::reg_for_depth(self.depth)
    }

    fn reg_for_depth(depth: usize) -> u8 {
        if depth < STACK_REG_COUNT {
            FIRST_STACK_REG + depth as u8
        } else {
            panic!(
                "Stack depth {} exceeds register count {}, spilling not yet implemented",
                depth, STACK_REG_COUNT
            );
        }
    }
}

impl Default for StackMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut stack = StackMachine::new();
        assert_eq!(stack.depth(), 0);

        let r1 = stack.push();
        assert_eq!(r1, 2);
        assert_eq!(stack.depth(), 1);

        let r2 = stack.push();
        assert_eq!(r2, 3);
        assert_eq!(stack.depth(), 2);

        let popped = stack.pop();
        assert_eq!(popped, 3);
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_peek() {
        let mut stack = StackMachine::new();
        stack.push();
        stack.push();
        stack.push();

        assert_eq!(stack.peek(0), 4);
        assert_eq!(stack.peek(1), 3);
        assert_eq!(stack.peek(2), 2);
    }
}
