pub trait Callback {
    fn on_some_info(&mut self, _msg: &str) {}
    fn on_some_warn(&mut self, _msg: &str) {}
    fn on_some_success(&mut self, _msg: &str) {}
    fn on_some_error(&mut self, _msg: &str) {}
    fn on_unknown_error(&mut self, _msg: &str) {}
}