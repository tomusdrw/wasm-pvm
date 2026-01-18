const FIRST_STACK_REG: u8 = 2;
const LAST_STACK_REG: u8 = 6;
const STACK_REG_COUNT: usize = (LAST_STACK_REG - FIRST_STACK_REG + 1) as usize;
/// Register used as temporary for spilled stack values (not part of operand stack r2-r6)
const SPILL_TEMP_REG: u8 = 7;

#[derive(Debug)]
pub struct StackMachine {
    depth: usize,
    max_depth: usize,
}

impl StackMachine {
    pub const fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 0,
        }
    }

    pub fn push(&mut self) -> u8 {
        let reg = Self::reg_for_depth(self.depth);
        self.depth += 1;
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
        }
        reg
    }

    pub fn pop(&mut self) -> u8 {
        assert!(self.depth > 0, "Stack underflow");
        self.depth -= 1;
        Self::reg_for_depth(self.depth)
    }

    #[allow(dead_code)]
    pub fn peek(&self, offset: usize) -> u8 {
        assert!(offset < self.depth, "Stack peek out of bounds");
        let idx = self.depth - 1 - offset;
        Self::reg_for_depth(idx)
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    pub fn reg_at_depth(depth: usize) -> u8 {
        if depth < STACK_REG_COUNT {
            FIRST_STACK_REG + depth as u8
        } else {
            SPILL_TEMP_REG // Use dedicated temp register for spilled values
        }
    }

    pub const fn needs_spill(depth: usize) -> bool {
        depth >= STACK_REG_COUNT
    }

    pub const fn spill_offset(depth: usize) -> i32 {
        let spill_index = depth - STACK_REG_COUNT;
        (spill_index as i32) * 8
    }

    fn reg_for_depth(depth: usize) -> u8 {
        if depth < STACK_REG_COUNT {
            FIRST_STACK_REG + depth as u8
        } else {
            SPILL_TEMP_REG // Use dedicated temp register for spilled values
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

    #[test]
    fn test_spill_depth() {
        let mut stack = StackMachine::new();
        // First 5 pushes use registers r2-r6
        for i in 0..5 {
            let reg = stack.push();
            assert_eq!(reg, 2 + i as u8);
        }
        assert_eq!(stack.depth(), 5);

        // 6th push spills to memory, uses dedicated temp register r7
        let reg = stack.push();
        assert_eq!(reg, SPILL_TEMP_REG); // r7, not r2!
        assert_eq!(stack.depth(), 6);

        let popped = stack.pop();
        assert_eq!(popped, SPILL_TEMP_REG); // r7
        assert_eq!(stack.depth(), 5);
    }

    #[test]
    fn test_needs_spill() {
        assert!(!StackMachine::needs_spill(0));
        assert!(!StackMachine::needs_spill(4));
        assert!(StackMachine::needs_spill(5));
        assert!(StackMachine::needs_spill(10));
    }
}
