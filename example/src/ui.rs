use std::collections::VecDeque;

use image::DynamicImage;
use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{
        Alignment::{self, Center},
        Constraint, Direction, Layout, Rect,
    },
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
    Frame,
};
use viuer::Config;

use crate::app::{App, Location};

pub fn draw_app(frame: &mut Frame<impl Backend>, app: &mut App) {
    let main_layout_chunks = create_main_layout(frame);

    draw_main_area(frame, main_layout_chunks[0]);

    let tabs_layout_chunks = create_tabs_layout(main_layout_chunks[0]);

    draw_tabs(app, frame, tabs_layout_chunks[0]);
    draw_current_tab(app, frame, tabs_layout_chunks[1]);

    draw_footer(frame, main_layout_chunks[1]);
}

fn create_main_layout(frame: &mut Frame<impl Backend>) -> Vec<Rect> {
    Layout::default()
        .margin(1)
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
        .split(frame.size())
}

fn draw_main_area(frame: &mut Frame<impl Backend>, chunk: Rect) {
    let top_title = Block::default()
        .borders(Borders::ALL)
        .title("Monsoon Weather Frog");

    frame.render_widget(top_title, chunk);
}

fn create_tabs_layout(chunk: Rect) -> Vec<Rect> {
    Layout::default()
        .horizontal_margin(3)
        .vertical_margin(2)
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(chunk)
}

fn draw_tabs(app: &mut App, frame: &mut Frame<impl Backend>, chunk: Rect) {
    let tab_titles = app
        .locations()
        .iter()
        .map(|l| {
            let (first, rest) = l.title().split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(rest, Style::default()),
            ])
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title("Locations"))
        .select(app.current_location_idx())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    frame.render_widget(tabs, chunk);
}

fn draw_current_tab(app: &mut App, frame: &mut Frame<impl Backend>, chunk: Rect) {
    if app.current_location().is_loaded() {
        draw_current_tab_weather(app, frame, chunk);
    } else {
        draw_current_tab_loading(frame, chunk);
    }
}

fn draw_current_tab_weather(app: &mut App, frame: &mut Frame<impl Backend>, chunk: Rect) {
    // Draw the table manually instead of using the Table widget because of sixel images.

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(3)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(chunk);

    let headers = [
        "",
        "Night",
        "Morning",
        "Afternoon",
        "Evening",
        "Max/min temp. (°C)",
        "Precip. (mm)",
        "Wind (m/s)",
    ];

    let mut rows = get_rows_from_location(app.current_location());
    rows.push_front(headers.map(Into::into).into());

    let cell_constrains = [
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
    ]
    .as_ref();

    for (row_idx, row) in rows.iter().enumerate() {
        let row_column_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(cell_constrains)
            .split(row_chunks[row_idx]);

        for (column_idx, column_text) in row.iter().enumerate() {
            if app.use_images() && row_idx > 0 && (1..=4).contains(&column_idx) {
                if let Some(image) = app.image_by_name(column_text) {
                    let block = Block::default().borders(Borders::NONE);
                    let image = Image::new(Some(block), image);
                    frame.render_widget(image, row_column_chunks[column_idx]);
                } else {
                    let paragraph = Paragraph::new(&**column_text).alignment(Alignment::Left);
                    frame.render_widget(paragraph, row_column_chunks[column_idx]);
                }
            } else {
                let mut style = Style::default().fg(Color::White);
                if row_idx == 0 || column_idx == 0 {
                    style = style.add_modifier(Modifier::BOLD);
                }
                let span = Span::styled(column_text, style);
                let paragraph = Paragraph::new(span).alignment(Alignment::Left);
                frame.render_widget(paragraph, row_column_chunks[column_idx]);
            }
        }
    }
}

fn get_rows_from_location(location: &Location) -> VecDeque<Vec<String>> {
    let data = location.forecast();

    data.iter()
        .map(|row| {
            row.iter()
                .cloned()
                .map(Option::unwrap_or_default)
                .collect::<Vec<String>>()
        })
        .collect()
}

fn draw_current_tab_loading(frame: &mut Frame<impl Backend>, chunk: Rect) {
    let text = Span::styled("Loading...", Style::default().add_modifier(Modifier::BOLD));

    let layout = Layout::default()
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(chunk);

    let paragraph = Paragraph::new(text).alignment(Center);
    frame.render_widget(paragraph, layout[0]);
}

fn draw_footer(frame: &mut Frame<impl Backend>, chunk: Rect) {
    let add_span = Spans::from(vec![
        Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Toggle images"),
        Span::raw("          "),
        Span::styled("←→", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Move to the next / prev location"),
        Span::raw("          "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Quit"),
    ]);

    let actions = Paragraph::new(add_span).alignment(Center);
    frame.render_widget(actions, chunk);
}

struct Image<'a> {
    block: Option<Block<'a>>,
    image: &'a DynamicImage,
}

impl<'a> Image<'a> {
    fn new(block: Option<Block<'a>>, image: &'a DynamicImage) -> Self {
        Self { block, image }
    }
}

impl<'a> Widget for Image<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }

        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let conf = Config {
            transparent: true,
            x: area.x,
            y: area.y as i16,
            ..Default::default()
        };

        viuer::print(self.image, &conf).expect("Image printing failed.");

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.get_mut(x, y).set_skip(true);
            }
        }
    }
}
