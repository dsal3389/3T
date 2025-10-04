use std::pin::Pin;
use std::sync::LazyLock;

use crate::app::Context;
use crate::app::Mode;
use crate::bind::Binder;
use crate::event::KeyCode;

pub static NORMAL_MODE_COMBOS: LazyLock<Binder> = LazyLock::new(|| {
    let mut combos = Binder::new();
    combos.add([KeyCode::Char('i'); 1], change_to_insert_mode);
    combos
});

fn change_to_insert_mode<'a>(cx: Context<'a>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        cx.state.mode = Mode::Insert;
    })
}
