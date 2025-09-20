use std::pin::Pin;
use std::sync::LazyLock;

use crate::app::Context;
use crate::app::Mode;
use crate::combo::ComboRegister;
use crate::event::KeyCode;

pub static NORMAL_MODE_COMBOS: LazyLock<ComboRegister> = LazyLock::new(|| {
    let mut combos = ComboRegister::new();
    combos.add([KeyCode::Char('i'); 1], change_to_insert_mode);
    combos
});

fn change_to_insert_mode<'a>(cx: Context<'a>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        cx.state.mode = Mode::Insert;
    })
}
