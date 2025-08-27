use threet_storage::models::User;

use crate::views::ViewKind;

pub enum Event {
    Stdin(Vec<u8>),
    Resize((u16, u16)),

    Tick,

    /// set different view for the app, the app knows how to initialize
    /// the specifically requested view, the enum to pass the view itself because
    /// the event setter doesn't need to know how to create the view and we don't
    /// want to constraint to view to must implemnet `Send` and `Sync`
    SetView(ViewKind),

    /// allow setting the user from outside the application
    /// or from a view
    SetUser(User),
    Render,
}
