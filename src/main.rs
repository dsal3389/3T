use std::sync::Arc;
use ratatui::DefaultTerminal;

mod logging;
mod views;
mod event;
mod app;

use app::App;

fn setup_logger() {
    let logger = logging::AppLogger::from_path("bb.log").expect("couldn't create logger");
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Debug)).unwrap();
}

async fn run(terminal: DefaultTerminal) -> anyhow::Result<()> {
    let app = Arc::new(App::new());
    app.run(terminal).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger();

    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}
