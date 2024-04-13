// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
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
  desc:  String,
  date:  String,
}

fn main() {
  let jst = FixedOffset::east_opt(9 * 3600).unwrap();
  let data = init();
  debug!("Parsed data: {:#?}", data);
  let Ok(notice_time) = NaiveTime::parse_from_str(&data.settings.notice_time, "%H:%M") else {
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

fn init() -> Data {
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
  match &std::fs::read_to_string("data.toml") {
    Ok(s) => toml::from_str(s).unwrap(),
    Err(e) => {
      if e.kind() == ErrorKind::NotFound {
        panic!(
          "`data.toml` is not found. Try `cp data-sample.toml data.toml`\n({})",
          e
        );
      }
      panic!("{}", e);
    }
  }
}
