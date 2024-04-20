// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
use chrono::{FixedOffset, NaiveDate, NaiveTime, Timelike, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
  fs::File,
  io::{ErrorKind, Write},
  path::Path,
  sync::{Arc, RwLock},
};

#[tokio::main]
async fn send_message(
  env: &Env,
  title: String,
  embeds: &Embed,
) -> Result<(), Box<dyn std::error::Error>> {
  info!("Sending message '{}'", title);
  let client = reqwest::Client::new();
  let _resp = client
    .post(format!(
      "https://discord.com/api/channels/{}/messages",
      env.channel_id
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bot {}", env.discord_bot_token))
    .body(
      json!({
          "content": format!("{}{}",
              if env.disable_everyone{""} else {"@everyone "},
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

#[derive(Debug, Serialize, Deserialize)]
struct Env {
  port:              i64,
  discord_bot_token: String,
  channel_id:        String,
  disable_everyone:  bool,
  notice_time:       String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Event {
  title: String,
  date:  String,
}

fn main() {
  let jst = FixedOffset::east_opt(9 * 3600).unwrap();
  let Ok((env, events)) = init() else {
    std::process::exit(1);
  };
  let Ok(notice_time) = NaiveTime::parse_from_str(&env.notice_time, "%H:%M") else {
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
        &env,
        String::from("I remind you of upcoming events!"),
        // TODO: check whether it is empty
        &build_embed(&events.read().unwrap()),
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

fn init() -> Result<(Arc<Env>, Arc<RwLock<Vec<Event>>>), ()> {
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
  let env: Env = match &std::fs::read_to_string("env.toml") {
    Ok(s) => toml::from_str(s).unwrap(),
    Err(e) => {
      if e.kind() == ErrorKind::NotFound {
        error!(
          "`env.toml` is not found. Try `cp sample-env.toml env.toml`\n({})",
          e
        );
      } else {
        error!("{}", e);
      }
      return Err(());
    }
  };
  debug!("{}", toml::to_string(&env).unwrap());
  {
    if env.discord_bot_token.is_empty() {
      error!("`settings.discord_bot_token` is empty");
      return Err(());
    }
    if env.channel_id.is_empty() {
      error!("`settings.channel_id` is empty");
      return Err(());
    }
  }

  // initialize `events.toml` if it doesn't exist
  let events_file_path = Path::new("events.toml");
  if !events_file_path.exists() {
    let Ok(mut file) = File::create_new(&events_file_path) else {
      error!("Cannot find `events.toml` and failed to create it");
      return Err(());
    };
    writeln!(
      file,
      r#"# [[events]]
# title = "EventTitle"
# date = "YYYY-MM-DD" # ISO 8601
"#
    )
    .unwrap();
  }

  let events: Vec<Event> = match &std::fs::read_to_string(&events_file_path) {
    Ok(s) => toml::from_str(s).unwrap(),
    Err(e) => {
      if e.kind() == ErrorKind::NotFound {
        error!(
          "`events.toml` is not found. Try `cp sample-events.toml events.toml`\n({})",
          e
        );
      } else {
        error!("{}", e);
      }
      return Err(());
    }
  };

  Ok((Arc::new(env), Arc::new(RwLock::new(events))))
}
