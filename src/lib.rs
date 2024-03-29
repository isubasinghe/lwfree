use std::{
    ops::Index,
    sync::atomic::{AtomicPtr, Ordering},
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
    type Input: Clone;
    type Output: Clone;
    type Cas: CasDescriptor;
    type Cases: CasDescriptors<Self::Cas> + Clone;
    type ContentionMeasure: ContentionMeasure;
    fn generator(&self, op: &Self::Input, contention: &mut Self::ContentionMeasure) -> Self::Cases;
    fn wrap_up(
        &self,
        result: Result<(), usize>,
        performed: &Self::Cases,
        contention: &mut Self::ContentionMeasure,
    ) -> Result<Self::Output, ()>;
}

pub struct OperationRecordBox<LF: NormalisedLockFree> {
    val: AtomicPtr<OperationRecord<LF>>,
}
enum OperationState<LF: NormalisedLockFree> {
    PreCAS,
    ExecuteCas(LF::Cases),
    PostCAS(LF::Cases, Result<(), usize>),
    Completed(LF::Output),
}

pub struct OperationRecord<LF: NormalisedLockFree> {
    owner_tid: std::thread::ThreadId,
    input: LF::Input,
    state: OperationState<LF>,
}

// wait-free queue
struct HelpQueue<LF: NormalisedLockFree> {
    _lf: LF,
}

impl<LF> HelpQueue<LF>
where
    LF: NormalisedLockFree,
{
    // TODO: append based on appendix a
    pub fn enqueue(&self, help: *const OperationRecordBox<LF>) {
        let _ = help;
    }
    pub fn peek(&self) -> Option<*const OperationRecordBox<LF>> {
        todo!()
    }
    pub fn try_remove_front(&self, completed: *const OperationRecordBox<LF>) -> Result<(), ()> {
        let _ = completed;
        todo!()
    }
}

pub struct WaitFreeSimulator<LF: NormalisedLockFree> {
    lf: LF,
    help_queue: HelpQueue<LF>,
}

impl<LF: NormalisedLockFree> WaitFreeSimulator<LF>
where
    OperationRecord<LF>: Clone,
{
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

    // guarantees that on return orb is no longer in help queue
    fn help_op(&self, orb: &OperationRecordBox<LF>) {
        loop {
            let or = unsafe { &*orb.val.load(Ordering::SeqCst) };
            let updated_or = match &or.state {
                OperationState::PreCAS => {
                    let cas_list = self
                        .lf
                        .generator(&or.input, &mut LF::ContentionMeasure::new());
                    Box::new(OperationRecord {
                        owner_tid: or.owner_tid.clone(),
                        state: OperationState::ExecuteCas(cas_list),
                        input: or.input.clone(),
                    })
                }
                OperationState::ExecuteCas(cas_list) => {
                    let result = self.cas_execute(cas_list, &mut LF::ContentionMeasure::new());
                    Box::new(OperationRecord {
                        owner_tid: or.owner_tid.clone(),
                        state: OperationState::PostCAS(cas_list.clone(), result),
                        input: or.input.clone(),
                    })
                }
                OperationState::PostCAS(cas_list, res) => {
                    if let Ok(result) =
                        self.lf
                            .wrap_up(res.clone(), cas_list, &mut LF::ContentionMeasure::new())
                    {
                        Box::new(OperationRecord {
                            owner_tid: or.owner_tid.clone(),
                            state: OperationState::Completed(result),
                            input: or.input.clone(),
                        })
                    } else {
                        // restart from the generator
                        Box::new(OperationRecord {
                            owner_tid: or.owner_tid.clone(),
                            state: OperationState::PreCAS,
                            input: or.input.clone(),
                        })
                    }
                }
                OperationState::Completed(..) => {
                    // if this fails, the orb must have been removed already
                    let _ = self.help_queue.try_remove_front(orb);
                    return;
                }
            };

            let updated_or = Box::into_raw(updated_or);
            if orb
                .val
                .compare_exchange(
                    or as *const OperationRecord<_> as *mut OperationRecord<_>,
                    updated_or,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                // Never got shared so safe to drop
                let _ = unsafe { Box::from_raw(updated_or) };
            }
        }
    }
    fn help_first(&self) {
        if let Some(help) = self.help_queue.peek() {
            self.help_op(unsafe { &*help });
        }
    }
    pub fn run(&self, op: LF::Input) -> LF::Output {
        // fast path
        for retry in 0.. {
            let help = /* once in a while */true;
            if help {
                self.help_first();
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
        }

        let or = OperationRecordBox {
            val: AtomicPtr::new(Box::into_raw(Box::new(OperationRecord {
                owner_tid: std::thread::current().id(),
                input: op,
                state: OperationState::PreCAS,
            }))),
        };
        self.help_queue.enqueue(&or);
        loop {
            // Safety: ??
            // Need Hazard Pointers here
            let or = unsafe { &*or.val.load(Ordering::SeqCst) };
            if let OperationState::Completed(t) = &or.state {
                break t.clone();
            } else {
                self.help_first();
            }
        }
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
    // use super::*;
}
