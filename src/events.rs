use serde::{Deserialize, Serialize};
use std::{
  fs::OpenOptions,
  io::{ErrorKind, Write},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
  pub title: String,
  pub date:  String,
}

#[derive(Serialize, Deserialize)]
pub struct EventsFile {
  pub events: Vec<Event>,
}
impl EventsFile {
  pub fn to_string(&self) -> String {
    toml::to_string(self).unwrap()
  }
}

pub fn read_events() -> Result<EventsFile, ()> {
  match &std::fs::read_to_string("events.toml") {
    Ok(s) => Ok(toml::from_str(s).unwrap()),
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
  }
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
