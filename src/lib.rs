use std::{
    slice::SliceIndex,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

pub trait ContentionMeasure {
    fn new() -> Self;
    fn detected(&self) -> bool;
    fn detect(&mut self);
    fn reset(&mut self);
}

pub struct CounterContentionMeasure {
    countention_cnt: usize,
}

impl ContentionMeasure for CounterContentionMeasure {
    fn new() -> CounterContentionMeasure {
        CounterContentionMeasure { countention_cnt: 0 }
    }
    fn detected(&self) -> bool {
        self.countention_cnt >= 10
    }
    fn detect(&mut self) {
        self.countention_cnt += 1;
    }
    fn reset(&mut self) {
        self.countention_cnt = 0;
    }
}

pub trait NormalisedLockFree {
    type Input;
    type Output;
    type Descriptor: SliceIndex<usize>;
    type ContentionMeasure: ContentionMeasure;
    fn generator(&self, op: &Self::Input) -> Vec<Self::Descriptor>;
    fn execute(
        &self,
        cases: &[Self::Descriptor],
        contention: &mut Self::ContentionMeasure,
    ) -> Result<(), usize>;
    fn cleanup(&self);
}
pub struct Help {
    completed: AtomicBool,
    at: AtomicUsize,
}
// wait-free queue
struct HelpQueue {}

impl HelpQueue {
    pub fn add(&self, help: *const Help) {}
    pub fn peek(&self) -> Option<*const Help> {
        todo!()
    }
    pub fn try_remove_front(&self, completed: *const Help) {
        todo!()
    }
}

pub struct WaitFreeSimulator<LF: NormalisedLockFree> {
    lf: LF,
    help_queue: HelpQueue,
}

impl<LF: NormalisedLockFree> WaitFreeSimulator<LF> {
    fn help(&self) {
        if let Some(help) = self.help_queue.peek() {}
    }
    pub fn run(&self, op: LF::Input) -> LF::Output {
        if
        /* once in a while */
        false {
            self.help();
        }
        let mut contention = LF::ContentionMeasure::new();
        let cas = self.lf.generator(&op);
        match self.lf.execute(&cas[..], &mut contention) {
            Ok(()) => {
                self.lf.cleanup();
            }
            Err(cnt) => {
                // slow path
                let help = Help {
                    completed: AtomicBool::new(false),
                    at: AtomicUsize::new(cnt),
                };
                self.help_queue.add(&help);
                while !help.completed.load(Ordering::SeqCst) {
                    self.help();
                }
            }
        }
        unimplemented!()
    }
}
// in consuming crate
/*
struct LockFreeLinkedList<T> {
    t: T,
}
impl<T> NormalisedLockFree for LockFreeLinkedList<T> {
    fn pre_cas(&self) {}
    fn help_cas(&self) {}
    fn post_cas(&self) {}
}

struct WaitFreeLinkedList<T> {
    simulator: WaitFreeSimulator<LockFreeLinkedList<T>>,
}

impl<T> WaitFreeLinkedList<T> {} */

#[cfg(test)]
mod tests {
    use super::*;
}
