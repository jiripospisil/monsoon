use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use chrono::Timelike;
use chrono_tz::Tz;
use image::DynamicImage;
use itertools::{
    Itertools,
    MinMaxResult::{MinMax, NoElements, OneElement},
};
use monsoon::{body::TimeSeries, Monsoon, Response};
use parking_lot::RwLock;
use tokio::sync::mpsc::{self, Receiver, Sender};

enum ResponseStatus {
    Loaded(Response),
    NotLoaded,
    Error(String),
}

struct LocationInner {
    title: Cow<'static, str>,
    lat: f64,
    lon: f64,
    tz: Tz,
    response: RwLock<ResponseStatus>,
}

#[derive(Clone)]
pub struct Location {
    inner: Arc<LocationInner>,
}

impl Location {
    pub fn new(title: impl Into<Cow<'static, str>>, lat: f64, lon: f64, tz: Tz) -> Self {
        Self {
            inner: Arc::new(LocationInner {
                title: title.into(),
                lat,
                lon,
                tz,
                response: RwLock::new(ResponseStatus::NotLoaded),
            }),
        }
    }

    pub fn title(&self) -> &Cow<'static, str> {
        &self.inner.title
    }

    pub fn forecast(&self) -> Vec<VecDeque<Option<String>>> {
        match *self.inner.response.read() {
            ResponseStatus::Loaded(ref response) => format_forecast(self.inner.tz, response),
            _ => vec![],
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(*self.inner.response.read(), ResponseStatus::Loaded(_))
    }

    async fn load(&self, load_event_tx: Sender<()>) {
        if matches!(
            *self.inner.response.read(),
            ResponseStatus::Loaded(_) | ResponseStatus::Error(_)
        ) {
            return;
        }

        let inner = &self.inner;

        let response = match Monsoon::new("example https://github.com/jiripospisil/monsoon") {
            Ok(monsoon) => match monsoon.get(inner.lat, inner.lon).await {
                Ok(response) => ResponseStatus::Loaded(response),
                Err(err) => ResponseStatus::Error(err.to_string()),
            },
            Err(err) => ResponseStatus::Error(err.to_string()),
        };

        *inner.response.write() = response;
        _ = load_event_tx.send(()).await;
    }
}

fn format_forecast(tz: Tz, response: &Response) -> Vec<VecDeque<Option<String>>> {
    let body = response.body().expect("Properly formatted body");

    body.properties
        .timeseries
        .iter()
        .group_by(|timeseries| timeseries.time.with_timezone(&tz).date_naive())
        .into_iter()
        .take(7)
        .enumerate()
        .map(|(idx, (day, hours))| {
            let hours: Vec<_> = hours.collect();

            let mut row = VecDeque::new();
            pick_symbols_from_hours(&mut row, idx, &hours);
            max_min_temperature(&mut row, &hours);
            precipitation(&mut row, &hours);
            wind(&mut row, &hours);
            row.push_front(Some(day.format("%A, %-d %B").to_string()));

            row
        })
        .collect()
}

fn pick_symbols_from_hours(row: &mut VecDeque<Option<String>>, idx: usize, hours: &[&TimeSeries]) {
    for hour in hours {
        if [0, 6, 12, 18].contains(&hour.time.hour()) {
            row.push_back(
                hour.data
                    .next_6_hours
                    .as_ref()
                    .map(|next| next.summary.symbol_code.to_owned()),
            );
        }
    }

    // The first day may not start at 0
    if idx == 0 && row.len() != 4 {
        row.push_front(
            hours[0]
                .data
                .next_6_hours
                .as_ref()
                .map(|next| next.summary.symbol_code.to_owned()),
        );
    }

    for _ in row.len()..4 {
        row.push_front(None);
    }
}

fn max_min_temperature(row: &mut VecDeque<Option<String>>, hours: &[&TimeSeries]) {
    match hours
        .iter()
        .filter_map(|hour| hour.data.instant.details.air_temperature)
        .minmax()
    {
        NoElements => {}
        OneElement(one) => row.push_back(format!("{}° / {}°", one, one).into()),
        MinMax(min, max) => row.push_back(format!("{}° / {}°", max, min).into()),
    };
}

fn precipitation(row: &mut VecDeque<Option<String>>, hours: &[&TimeSeries]) {
    // This is not what yr.no does but good enough
    let total: f64 = hours
        .iter()
        .filter_map(|hour| {
            hour.data
                .next_6_hours
                .as_ref()?
                .details
                .as_ref()?
                .precipitation_amount
        })
        .sum();

    row.push_back(format!("{:.1}", total).into());
}

fn wind(row: &mut VecDeque<Option<String>>, hours: &[&TimeSeries]) {
    if let Some(max) = hours
        .iter()
        .filter_map(|hour| hour.data.instant.details.wind_speed)
        .reduce(f64::max)
    {
        row.push_back(format!("{:.1}", max).into());
    } else {
        row.push_back(None);
    }
}

pub struct App {
    locations: Vec<Location>,
    current_location_idx: usize,

    should_quit: bool,
    use_images: bool,

    load_event_tx: Sender<()>,
    load_event_rx: Receiver<()>,

    images: HashMap<&'static str, DynamicImage>,
}

impl App {
    pub fn new(
        locations: impl Into<Vec<Location>>,
        images: HashMap<&'static str, DynamicImage>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<()>(100);

        let s = Self {
            locations: locations.into(),
            current_location_idx: 0,
            should_quit: false,
            use_images: true,
            load_event_tx: tx,
            load_event_rx: rx,
            images,
        };

        s.ensure_loaded();
        s
    }

    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn current_location_idx(&self) -> usize {
        self.current_location_idx
    }

    pub fn current_location(&self) -> &Location {
        &self.locations[self.current_location_idx]
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn image_by_name(&self, name: &str) -> Option<&DynamicImage> {
        self.images.get(name)
    }

    pub async fn on_event(&mut self) {
        self.load_event_rx.recv().await;
    }

    pub fn on_key(&mut self, key: char) {
        match key {
            'q' => self.should_quit = true,
            'i' => self.use_images = !self.use_images,
            _ => {}
        };
    }

    pub fn on_left(&mut self) {
        self.current_location_idx = self.current_location_idx.saturating_sub(1);
        self.ensure_loaded();
    }

    pub fn on_right(&mut self) {
        self.current_location_idx = self
            .current_location_idx
            .saturating_add(1)
            .clamp(0, self.locations.len() - 1);
        self.ensure_loaded();
    }

    pub fn use_images(&self) -> bool {
        self.use_images
    }

    fn ensure_loaded(&self) {
        let location = self.current_location().clone();
        let tx = self.load_event_tx.clone();

        tokio::spawn(async move { location.load(tx).await });
    }
}
