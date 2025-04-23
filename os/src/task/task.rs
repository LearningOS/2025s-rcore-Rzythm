//! Types related to task management

use super::TaskContext;

/// SyscallCounter
#[derive(Copy, Clone)]
pub struct SyscallCounter {
    entries: [(usize, usize); 5],   
    len: usize,
}

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The syscall counte
    pub syscall_count: SyscallCounter,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

/// impl for SyscallCounter
impl SyscallCounter {
    /// initialized
    pub const fn new() -> Self {
        Self {
            entries: [(0, 0); 5],
            len: 0,
        }
    }

    /// increment
    pub fn increment(&mut self, syscall_id: usize) {
        for i in 0..self.len {
            if self.entries[i].0 == syscall_id {
                self.entries[i].1 += 1;
                return;
            }
        }
        if self.len < 5 {
            self.entries[self.len] = (syscall_id, 1);
            self.len += 1;
        }
    }

    /// get_count
    pub fn get_count(&self, syscall_id: usize) -> usize {
        for i in 0..self.len {
            if self.entries[i].0 == syscall_id {
                return self.entries[i].1;
            }
        }
        0
    }
}
