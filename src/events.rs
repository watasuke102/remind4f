use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
  fs::OpenOptions,
  io::{ErrorKind, Write},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
  pub title: String,
  pub date:  NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventFileEntry {
  pub title: String,
  pub date:  String,
}
#[derive(Serialize, Deserialize)]
pub struct EventsFile {
  pub events: Vec<EventFileEntry>,
}
impl EventsFile {
  pub fn to_string(&self) -> String {
    toml::to_string(self).unwrap()
  }
}

pub fn read_events() -> Result<Vec<Event>, ()> {
  let events: EventsFile = match &std::fs::read_to_string("events.toml") {
    Ok(s) => toml::from_str(s).unwrap(),
    Err(e) => {
      if e.kind() == ErrorKind::NotFound {
        println!(
          "ERROR: `events.toml` is not found. Try `cp sample-events.toml events.toml`\n({})",
          e
        );
      } else {
        println!("ERROR: {}", e);
      }
      return Err(());
    }
  };
  let mut events: Vec<Event> = events
    .events
    .into_iter()
    .flat_map(|event| {
      let Ok(event_date) = NaiveDate::parse_from_str(&event.date, "%F") else {
        println!("ERROR: Failed to parse event date: {:?}", event);
        return None;
      };

      Some(Event {
        title: event.title,
        date:  event_date,
      })
    })
    .collect();
  events.sort_by(|a, b| a.date.cmp(&b.date));
  Ok(events)
}

pub fn write(text: String) -> Result<(), ()> {
  let Ok(mut file) = OpenOptions::new()
    .write(true)
    .create(true)
    .open("events.toml")
  else {
    println!("ERROR: Cannot find `events.toml` and failed to create it");
    return Err(());
  };
  match writeln!(file, "{}", text) {
    Ok(_) => Ok(()),
    Err(_) => Err(()),
  }
}
