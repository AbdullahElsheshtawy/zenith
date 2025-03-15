use super::context::RenderContext;

pub struct DeletionQueue<'a> {
    deletors: Vec<Box<dyn FnOnce(&mut RenderContext) + 'a>>,
}

impl<'a> DeletionQueue<'a> {
    pub fn new() -> Self {
        Self {
            deletors: Vec::new(),
        }
    }

    pub fn push(&mut self, function: Box<dyn FnOnce(&mut RenderContext) + 'a>) {
        self.deletors.push(function);
    }

    pub fn flush(&mut self, rcx: &mut RenderContext) {
        while let Some(deletor) = self.deletors.pop() {
            deletor(rcx)
        }
    }
}
