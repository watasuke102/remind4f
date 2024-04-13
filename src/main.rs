// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::ErrorKind;

#[tokio::main]
async fn send_message(
  settings: &Settings,
  title: String,
  embeds: &Embed,
) -> Result<(), Box<dyn std::error::Error>> {
  info!("Sending message '{}'", title);
  let client = reqwest::Client::new();
  let _resp = client
    .post(format!(
      "https://discord.com/api/channels/{}/messages",
      settings.channel_id
    ))
    .header("Content-Type", "application/json")
    .header(
      "Authorization",
      format!("Bot {}", settings.discord_bot_token),
    )
    .body(
      json!({
          "content": format!("{}{}",
              if settings.disable_everyone{""} else {"@everyone\n"},
              title
          ),
          "tts": false,
          "embeds": [embeds]
      })
      .to_string(),
    )
    .send()
    .await?;
  Ok(())
}

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
      match send_message(
        &settings,
        String::from("I remind you of upcoming events!"),
        &embed,
      ) {
        Ok(_) => info!("The message was sent"),
        Err(e) => error!("Something went wrong when sending the message: {:#?}", e),
      }
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
  {
    if data.settings.discord_bot_token.is_empty() {
      error!("`settings.discord_bot_token` is empty");
      return Err(());
    }
    if data.settings.channel_id.is_empty() {
      error!("`settings.channel_id` is empty");
      return Err(());
    }
  }
  Ok((data.settings, build_embed(&data.events)))
}
