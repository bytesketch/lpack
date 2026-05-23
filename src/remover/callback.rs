#[allow(dead_code)]
pub trait RemoverCallback {
    fn on_some_info(&mut self, _msg: &str) {}
    fn on_some_warn(&mut self, _msg: &str) {}
    fn on_some_success(&mut self, _msg: &str) {}
    fn on_some_error(&mut self, _msg: &str) {}
    fn on_unknown_error(&mut self, _msg: &str) {}
    fn prompt_string(&mut self, _msg: &str) -> String {
        String::new()
    }

    fn prompt_confirm(&mut self, _msg: &str, default: bool) -> bool {
        default
    }
}
