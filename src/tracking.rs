use json::{json, Value};
use std::collections::HashMap;

const MIN_PER_HOUR: usize = 60;
const HOURS_PER_DAY: usize = 24;
const DAYS_PER_WEEK: usize = 7;

/// Tracking and aggregation of Tweet History
pub struct Tracking {
    /// History of last hour per day
    one_hour: Vec<Vec<u64>>,
    /// History of last days per hour
    one_day: Vec<Vec<u64>>,
    /// History of last week per day
    one_week: Vec<Vec<u64>>,
    /// Moving average and trend per minute over one hour
    trend_by_minute: Vec<(f64, f64, u64)>,
    /// Moving average and trend per hour over one day
    trend_by_hour: Vec<(f64, f64, u64)>,
    /// Moving average and trend per day over one week
    trend_by_day: Vec<(f64, f64, u64)>,
    // Counters for minutes, day, time
    minute_count: usize,
    hour_count: usize,
    day_count: usize,
    minute_level: u64,
    hour_level: u64,
    day_level: u64,
    // Key to index table
    key_index: HashMap<String, usize>,
}

impl ToString for Tracking {
    fn to_string(&self) -> String {
        let items: Vec<Value> = self
            .key_index
            .iter()
            .map(|(k, &i)| {
                json!({
                        "name": k.to_owned(),
                        "curent_hour": self.one_hour[i].to_owned(),
                        "current_day": self.one_day[i].to_owned(),
                        "current_week": self.one_week[i].to_owned(),
                        "trend_by_minute": self.trend_by_minute[i],
                        "trend_by_hour": self.trend_by_hour[i],
                        "trend_by_hour": self.trend_by_hour[i],
                })
            })
            .collect();
        json::to_string(&items).expect("json")
    }
}

impl Tracking {
    pub fn new(keys: &Vec<String>) -> Self {
        let mut trend_by_minute = Vec::with_capacity(keys.len());
        let mut trend_by_hour = Vec::with_capacity(keys.len());
        let mut trend_by_day = Vec::with_capacity(keys.len());
        trend_by_minute.resize(keys.len(), (0f64, 0f64, 0u64));
        trend_by_hour.resize(keys.len(), (0f64, 0f64, 064));
        trend_by_day.resize(keys.len(), (0f64, 0f64, 064));
        Tracking {
            one_hour: Tracking::init_buffer(keys.len(), MIN_PER_HOUR),
            one_day: Tracking::init_buffer(keys.len(), HOURS_PER_DAY),
            one_week: Tracking::init_buffer(keys.len(), DAYS_PER_WEEK),
            trend_by_minute: trend_by_minute,
            trend_by_hour: trend_by_hour,
            trend_by_day: trend_by_day,
            minute_count: 0,
            hour_count: 0,
            day_count: 0,
            minute_level: 1,
            hour_level: 1,
            day_level: 1,
            key_index: Tracking::init_key_table(keys),
        }
    }

    /// Increments the number of tweet for a track key.
    /// Although very unlikely to overflow, using saturating_add.
    pub fn increment_count_by_key(&mut self, key: &str) {
        if let Some(&i) = self.key_index.get(key) {
            self.increment_count_by_index(i);
        };
    }

    /// Increments the number of tweet for a track index.
    /// Although very unlikely to overflow, using saturating_add.
    pub fn increment_count_by_index(&mut self, index: usize) {
        if index < self.key_index.len() {
            let m = self.minute_count;
            let h = self.hour_count;
            let d = self.day_count;
            self.one_hour
                .get_mut(index)
                .map(|v| v[m] = v[m].saturating_add(1));
            self.one_day
                .get_mut(index)
                .map(|v| v[h] = v[h].saturating_add(1));
            self.one_week
                .get_mut(index)
                .map(|v| v[d] = v[d].saturating_add(1));
        };
    }

    /// Function to be called every minute to process data
    /// Basically agregating current counts into minute slots.
    /// Also handling minute/hour/day overflows and calculating
    /// moving avarage.
    pub fn minute_complete(&mut self) {
        let previous_minute = self.minute_count;
        let next_minute = (self.minute_count + 1) % MIN_PER_HOUR;
        let minute_level = self.minute_level;

        // Clearing oldest minute and getting differens in count
        let change: Vec<(u64, u64)> = self
            .one_hour
            .iter_mut()
            .map(|minutes| {
                let update = (minutes[previous_minute], minutes[next_minute]);
                minutes[next_minute] = 0;
                update
            })
            .collect();

        for (i, (mav, trend, sum)) in self.trend_by_minute.iter_mut().enumerate() {
            let (new, old) = change[i];
            *sum += new - old;
            *mav = *sum as f64 / minute_level as f64;
            if *mav != 0f64 {
                *trend = ((new as f64 / minute_level as f64) - *mav) / *mav * 100f64;
            }
        }

        self.minute_count = next_minute;
        if minute_level < MIN_PER_HOUR as u64 {
            self.minute_level += 1;
        }
        if next_minute == 0 {
            self.hour_complete();
        }
    }

    fn hour_complete(&mut self) {
        let previous_hour = self.hour_count;
        let next_hour = (self.hour_count + 1) % HOURS_PER_DAY;
        let hour_level = self.hour_level;
        // Clearing oldest hour and getting differens in count
        let change: Vec<(u64, u64)> = self
            .one_day
            .iter_mut()
            .map(|hours| {
                let update = (hours[previous_hour], hours[next_hour]);
                hours[next_hour] = 0;
                update
            })
            .collect();

        for (i, (mav, trend, sum)) in self.trend_by_hour.iter_mut().enumerate() {
            let (new, old) = change[i];
            *sum += new - old;
            *mav = *sum as f64 / hour_level as f64;
            if *mav != 0f64 {
                *trend = ((new as f64 / hour_level as f64) - *mav) / *mav * 100f64;
            }
        }

        self.hour_count = next_hour;
        if hour_level < HOURS_PER_DAY as u64 {
            self.hour_level += 1;
        }
        if next_hour == 0 {
            self.day_complete();
        }
    }

    fn day_complete(&mut self) {
        let previous_day = self.day_count;
        let next_day = (self.day_count + 1) % DAYS_PER_WEEK;
        let day_level = self.day_level;
        // Clearing oldest day and getting differens in count
        let change: Vec<(u64, u64)> = self
            .one_week
            .iter_mut()
            .map(|days| {
                let update = (days[previous_day], days[next_day]);
                days[next_day] = 0;
                update
            })
            .collect();

        for (i, (mav, trend, sum)) in self.trend_by_day.iter_mut().enumerate() {
            let (new, old) = change[i];
            *sum += new - old;
            *mav = *sum as f64 / day_level as f64;
            if *mav != 0f64 {
                *trend = ((new as f64 / day_level as f64) - *mav) / *mav * 100f64;
            }
        }

        self.day_count = next_day;
        if self.day_level < DAYS_PER_WEEK as u64 {
            self.day_level += 1;
        }
    }

    fn init_buffer(key_len: usize, data_len: usize) -> Vec<Vec<u64>> {
        (0..key_len)
            .map(|_| {
                let mut buffer = Vec::with_capacity(data_len);
                buffer.resize(data_len, 0);
                buffer
            })
            .collect()
    }

    fn init_key_table(keys: &Vec<String>) -> HashMap<String, usize> {
        keys.iter()
            .enumerate()
            .map(|(i, key)| (key.to_owned(), i))
            .collect()
    }
}

#[test]
fn create() {
    let tracking = Tracking::new(&vec![
        "art".to_string(),
        "music".to_string(),
        "photography".to_string(),
        "love".to_string(),
        "fashion".to_string(),
    ]);
    assert_eq!(tracking.key_index.len(), 5);
    assert_eq!(tracking.one_hour.len(), 5);
    assert_eq!(tracking.one_day.len(), 5);
    assert_eq!(tracking.one_week.len(), 5);
    assert_eq!(tracking.trend_by_minute.len(), 5);
    assert_eq!(tracking.trend_by_hour.len(), 5);
    assert_eq!(tracking.trend_by_day.len(), 5);

    assert_eq!(tracking.one_hour[0].len(), MIN_PER_HOUR);
    assert_eq!(tracking.one_day[0].len(), HOURS_PER_DAY);
    assert_eq!(tracking.one_week[0].len(), DAYS_PER_WEEK);

    assert_eq!(tracking.one_hour[4].len(), MIN_PER_HOUR);
    assert_eq!(tracking.one_day[4].len(), HOURS_PER_DAY);
    assert_eq!(tracking.one_week[4].len(), DAYS_PER_WEEK);
}

#[test]
fn to_string() {
    let tracking = Tracking::new(&vec!["art".to_string()]);
    assert_eq!(tracking.to_string(), "[{\"curent_hour\":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],\"current_day\":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],\"current_week\":[0,0,0,0,0,0,0],\"name\":\"art\",\"trend_by_hour\":[0.0,0.0,64],\"trend_by_minute\":[0.0,0.0,0]}]");
}

#[test]
fn new_tweet_by_topic() {
    let mut tracking = Tracking::new(&vec!["art".to_string()]);
    tracking.increment_count_by_key("art");
    assert_eq!(tracking.one_hour[0][0], 1);
}

#[test]
fn new_tweet_by_index() {
    let mut tracking = Tracking::new(&vec!["art".to_string()]);
    tracking.increment_count_by_index(0);
    assert_eq!(tracking.one_hour[0][0], 1);
}

#[test]
fn next_minute() {
    let mut tracking = Tracking::new(&vec!["art".to_string()]);
    tracking.increment_count_by_index(0);
    tracking.increment_count_by_index(0);
    tracking.minute_complete();
    tracking.increment_count_by_index(0);
    assert_eq!(tracking.one_hour[0][0], 2);
    assert_eq!(tracking.one_hour[0][1], 1);
    assert_eq!(tracking.trend_by_minute[0], (2f64, 0f64, 2u64));
    tracking.minute_complete();
    assert_eq!(tracking.trend_by_minute[0], (1.5f64, -66.66666666666666, 3u64));
}
