// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use chrono::{FixedOffset, NaiveTime, Timelike, Utc};
use log::{debug, error, info, trace};
use serde::Deserialize;
use std::io::ErrorKind;

#[derive(Deserialize, Debug)]
struct Data {
  settings: Settings,
  events:   Vec<Event>,
}
#[derive(Deserialize, Debug)]
struct Settings {
  discord_bot_token: String,
  channel_id:        String,
  disable_everyone:  bool,
  notice_time:       String,
}
#[derive(Deserialize, Debug)]
struct Event {
  title: String,
  date:  String,
}

fn main() {
  let jst = FixedOffset::east_opt(9 * 3600).unwrap();
  let Ok((settings, embed)) = init() else {
    std::process::exit(1);
  };
  debug!("Embed: {:#?}", embed);
  let Ok(notice_time) = NaiveTime::parse_from_str(&settings.notice_time, "%H:%M") else {
    error!("Failed to parse `notice_time`; please check data.toml");
    std::process::exit(1);
  };

  info!("Bot is ready");
  loop {
    debug!("tick");
    let now = Utc::now().with_timezone(&jst);
    if now.time().hour() == notice_time.hour() && now.time().minute() == notice_time.minute() {
      info!("On time!");
    }
    std::thread::sleep(core::time::Duration::from_millis(1000 * 60));
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Embed {
  title:  String,
  color:  u32,
  fields: Vec<Field>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Field {
  name:  String,
  value: String,
}

fn build_embed(events: &Vec<Event>) -> Embed {
  let today = Utc::now()
    .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
    .date_naive();
  let mut result = Embed {
    title:  "Events".to_string(),
    color:  0x98c379,
    fields: Vec::<Field>::new(),
  };
  for event in events {
    let Ok(event_date) = NaiveDate::parse_from_str(&event.date, "%F") else {
      error!("Failed to parse event date: {:?}", event);
      continue;
    };
    if event_date < today {
      info!("Overdue event: {:?}", event);
      continue;
    }
    let days = (event_date - today).num_days();
    result.fields.push(Field {
      name:  event.title.clone(),
      value: format!("Due: {} day{}", days, if days == 1 { "" } else { "s" }),
    })
  }
  result
}

fn init() -> Result<(Settings, Embed), ()> {
  {
    use simplelog::*;
    CombinedLogger::init(vec![
      TermLogger::new(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
      ),
      WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        std::fs::File::create("remind4f.log").unwrap(),
      ),
    ])
    .unwrap();
  }
  let data: Data = match &std::fs::read_to_string("data.toml") {
    Ok(s) => toml::from_str(s).unwrap(),
    Err(e) => {
      if e.kind() == ErrorKind::NotFound {
        error!(
          "`data.toml` is not found. Try `cp data-sample.toml data.toml`\n({})",
          e
        );
      } else {
        error!("{}", e);
      }
      return Err(());
    }
  };
  Ok((data.settings, build_embed(&data.events)))
}
