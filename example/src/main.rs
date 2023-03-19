use app::App;
use app::Location;
use chrono_tz::Asia::Bangkok;
use chrono_tz::Europe::Prague;
use crossterm::event::EventStream;
use crossterm::terminal;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};
use futures::StreamExt;
use image::io::Reader;
use image::DynamicImage;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::io;
use std::io::Cursor;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let _result = main_loop(&mut terminal).await;

    terminal::disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn main_loop(terminal: &mut Terminal<impl Backend>) -> Result<()> {
    let mut app = create_app();
    let mut event_stream = EventStream::new();

    loop {
        terminal.draw(|frame| ui::draw_app(frame, &mut app))?;

        tokio::select! {
            event = event_stream.next() => {
                match event {
                    Some(Ok(event)) => if let Event::Key(c) = event { match c.code {
                            KeyCode::Left => {
                                app.on_left();
                                // Force redraw because of images
                                _ = terminal.clear();
                            }
                            KeyCode::Right =>{
                                app.on_right();
                                // ditto
                                _ = terminal.clear();
                            },
                            KeyCode::Char(c) => {
                                app.on_key(c);
                                // ditto
                                _ = terminal.clear();
                            }
                            _ => {}
                    }},

                    Some(Err(err)) => panic!("{}", err),
                    None => panic!("None?"),
                };
            },

            _ = app.on_event() => {},
        }

        if app.should_quit() {
            return Ok(());
        }
    }
}

static ICONS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/icons");

fn create_app() -> App {
    let images = ICONS
        .entries()
        .iter()
        .filter_map(|icon| {
            let file = icon.as_file()?;

            (
                file.path().file_stem()?.to_str()?,
                load_image(file.contents()),
            )
                .into()
        })
        .collect::<HashMap<_, _>>();

    App::new(
        [
            Location::new("Prague, Czech Republic", 50.0880, 14.4207, Prague),
            Location::new("Bangkok, Thailand", 13.7540, 100.5014, Bangkok),
        ],
        images,
    )
}

fn load_image(bytes: &'static [u8]) -> DynamicImage {
    Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
}
