mod channel;
mod user;

pub use channel::Channel;
pub use user::User;

pub(crate) trait Model: Send {
    /// returns the table name for current item
    fn table_name() -> &'static str;

    /// returns the querable fields for the current item
    fn fields() -> Vec<String>;
}
