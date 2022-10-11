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
    type CasDescriptor;
    type ContentionMeasure: ContentionMeasure;
    fn prepare(&self, op: &Self::Input) -> Vec<Self::CasDescriptor>;
    fn execute(&self, cases: &[Self::CasDescriptor], contention: &mut Self::ContentionMeasure) -> Result<(), usize>;
    fn cleanup(&self);
}
pub struct Help {}
// wait-free queue
struct HelpQueue {

}

impl HelpQueue {
    pub fn add(&self, help: Help) {
    }
    pub fn peek(&self) -> Option<&Help> {
        todo!()
    }
    pub fn try_remove_front(&self, completed: &Help) {
        todo!()
    }
}

pub struct WaitFreeSimulator<LF: NormalisedLockFree> {
    lf: LF,
    help_queue: HelpQueue,
}

impl<LF: NormalisedLockFree> WaitFreeSimulator<LF> {
    pub fn run(&self, op: LF::Input) -> LF::Output {
        if /* once in a while */ false {
            if let Some(help) = self.help_queue.peek() {
                // do something to help
                // help `help` make progress


            }
        }
        let mut contention = LF::ContentionMeasure::new();
        let cas = self.lf.prepare(&op);
        match self.lf.execute(&cas[..], &mut contention) {
            Ok(()) => {
                self.lf.cleanup();
            }, 
            Err(cnt) => {
                // slow path 
                let help = Help {};
                self.help_queue.add(help);

                
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
