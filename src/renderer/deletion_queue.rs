use super::context::RenderContext;

type DeletableFn<'a> = Box<dyn FnOnce(&mut RenderContext) + 'a>;
pub struct DeletionQueue<'a> {
    deletors: Vec<DeletableFn<'a>>,
}

impl<'a> DeletionQueue<'a> {
    pub fn new() -> Self {
        Self {
            deletors: Vec::new(),
        }
    }

    pub fn push(&mut self, function: DeletableFn<'a>) {
        self.deletors.push(function);
    }

    pub fn flush(&mut self, rcx: &mut RenderContext) {
        while let Some(deletor) = self.deletors.pop() {
            deletor(rcx)
        }
    }
}
