#[derive(Clone, Debug, Default)]
pub struct PreviewTracker<T> {
    snapshot: Option<T>,
}

impl<T: Clone> PreviewTracker<T> {
    pub fn new() -> Self {
        Self { snapshot: None }
    }

    pub fn begin(&mut self, state: &T) {
        if self.snapshot.is_none() {
            self.snapshot = Some(state.clone());
        }
    }

    pub fn is_active(&self) -> bool {
        self.snapshot.is_some()
    }

    pub fn snapshot(&self) -> Option<&T> {
        self.snapshot.as_ref()
    }

    pub fn end(&mut self) -> Option<T> {
        self.snapshot.take()
    }
}
