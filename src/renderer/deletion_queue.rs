use super::context::RenderContext;

type DeletableFn = Box<dyn FnOnce(&mut RenderContext)>;
pub struct DeletionQueue {
    deletors: Vec<DeletableFn>,
}

impl DeletionQueue {
    pub fn new() -> Self {
        Self {
            deletors: Vec::new(),
        }
    }

    pub fn push(&mut self, function: DeletableFn) {
        self.deletors.push(function);
    }

    pub fn flush(&mut self, rcx: &mut RenderContext) {
        while let Some(deletor) = self.deletors.pop() {
            deletor(rcx)
        }
    }
}
