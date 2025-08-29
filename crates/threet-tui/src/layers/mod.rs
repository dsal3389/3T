/// a layers is like a view, but it is under view, it renders smaller stuff
/// on top of the view, like dialog box messages etc
pub trait Layer {
    async fn handle_key(&mut self, key: char) -> bool;

    fn render(&self);
}
