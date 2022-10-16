use std::{
    ops::Index,
    slice::SliceIndex,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

const CONTENTION_THRESHOLD: usize = 2;
const RETRY_THRESHOLD: usize = 2;

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
        self.countention_cnt > CONTENTION_THRESHOLD
    }
    fn detect(&mut self) {
        self.countention_cnt += 1;
    }
    fn reset(&mut self) {
        self.countention_cnt = 0;
    }
}

pub trait CasDescriptor {
    fn execute(&self) -> Result<(), ()>;
}

pub trait CasDescriptors<D>: Index<usize, Output = D>
where
    D: CasDescriptor,
{
    fn len(&self) -> usize;
}

pub trait NormalisedLockFree {
    type Input;
    type Output;
    type Cas: CasDescriptor;
    type Cases: CasDescriptors<Self::Cas>;
    type ContentionMeasure: ContentionMeasure;
    fn generator(&self, op: &Self::Input, contention: &mut Self::ContentionMeasure) -> Self::Cases;
    fn wrap_up(
        &self,
        result: Result<(), usize>,
        performed: &Self::Cases,
        contention: &mut Self::ContentionMeasure,
    ) -> Result<Self::Output, ()>;
}
pub struct OperationRecord {
    completed: AtomicBool,
    at: AtomicUsize,
}
// wait-free queue
struct HelpQueue {}

impl HelpQueue {
    pub fn add(&self, help: *const OperationRecord) {}
    pub fn peek(&self) -> Option<*const OperationRecord> {
        todo!()
    }
    pub fn try_remove_front(&self, completed: *const OperationRecord) {
        todo!()
    }
}

pub struct WaitFreeSimulator<LF: NormalisedLockFree> {
    lf: LF,
    help_queue: HelpQueue,
}

impl<LF: NormalisedLockFree> WaitFreeSimulator<LF> {
    fn cas_execute<C: ContentionMeasure>(
        &self,
        descriptors: &LF::Cases,
        contention: &mut C,
    ) -> Result<(), usize> {
        let len = descriptors.len();
        for i in 0..len {
            if descriptors[i].execute().is_err() {
                contention.detect();
                return Err(i);
            }
        }
        Ok(())
    }
    fn help(&self) {
        if let Some(help) = self.help_queue.peek() {}
    }
    pub fn run(&self, op: LF::Input) -> LF::Output {
        // fast path
        for retry in 0.. {
            let help = /* once in a while */ false;
            if help {
                self.help();
            }
            let mut contention = LF::ContentionMeasure::new();
            if contention.detected() {
                break;
            }
            let cas = self.lf.generator(&op, &mut contention);
            if contention.detected() {
                break;
            }
            let result = self.cas_execute(&cas, &mut contention);
            if contention.detected() {
                break;
            }
            match self.lf.wrap_up(result, &cas, &mut contention) {
                Ok(outcome) => return outcome,
                Err(()) => {}
            };
            if contention.detected() {}

            if retry > RETRY_THRESHOLD {
                // slow  path
                break;
            }

            /* if let Err(cnt) = result {
                // slow path
                let help = Help {
                    completed: AtomicBool::new(false),
                    at: AtomicUsize::new(cnt),
                };
                self.help_queue.add(&help);
                while !help.completed.load(Ordering::SeqCst) {
                    self.help();
                }
            } */
        }

        let help = OperationRecord {
            completed: AtomicBool::new(false),
            at: AtomicUsize::new(0),
        };
        self.help_queue.add(&help);
        while !help.completed.load(Ordering::SeqCst) {
            self.help();
        }
        unreachable!()
    }
}

/**
 * ABA problem
 * head -> A (@0x1)
 * insert B (@0x2); B.next = 0x1
 * CAS(head, 0x1, 0x2)
 * succeed if A is still at the head
 * fail if A is no longer at the head
 * imagine above has not yet executed
 * meanwhile
 * insert C (@0x3) C.next = 0x1
 * CAS(head, 0x1, 0x3)
 * remove A
 * CAS(C.next, 0x1, 0x0)
 * insert D (@0x1); D.next = 0x3
 * CAS(head, 0x3, 0x1)
 * head -> D(@0x1) -> C(@0x3) -> .
 * CAS(head, 0x1, 0x2)
 * head -> B(@0x2) -> A(0x1) actually D(@0x1) -> C(@0x3) -> .
 * It works out here due to A and D having the same address, but if we imagine a doubly linked list it doesn't
 * This CAS should have failed
 * We need a counter associated with any given field
 */

#[cfg(test)]
mod tests {
    use super::*;
}
