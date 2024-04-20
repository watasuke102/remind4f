// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serenity::{
  all::{Context, EventHandler, GatewayIntents, GuildId, Interaction, Ready},
  async_trait,
  prelude::TypeMapKey,
  Client,
};
use std::{
  fs::File,
  io::{ErrorKind, Write},
  path::Path,
  sync::Arc,
};
mod show;

#[derive(Debug, Serialize, Deserialize)]
pub struct Env {
  pub port:              i64,
  pub discord_bot_token: String,
  pub channel_id:        u64,
  pub disable_everyone:  bool,
  pub notice_time:       String,
}
impl TypeMapKey for Env {
  type Value = Arc<Env>;
}
#[derive(Debug, Serialize, Deserialize)]
struct Event {
  title: String,
  date:  String,
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, ready: Ready) {
    let data = ctx.data.read().await;
    let Some(env) = data.get::<Env>() else {
      return;
    };
    GuildId::new(env.channel_id)
      .set_commands(&ctx.http, vec![show::register()])
      .await;
    info!(
      "ready> name: {}, version: {}",
      ready.user.name, ready.version
    );
  }
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    let data = ctx.data.read().await;
    let Some(env) = data.get::<Env>() else {
      return;
    };
    let Interaction::Command(command) = interaction else {
      return;
    };
    match command.data.name.as_str() {
      "show" => show::execute(&env, &ctx, &command).await,
      _ => (),
    };
  }
}

#[tokio::main]
async fn main() {
  let Ok(env) = init() else {
    std::process::exit(1);
  };

  let Ok(mut client) = Client::builder(&env.discord_bot_token, GatewayIntents::empty())
    .event_handler(Handler)
    .await
  else {
    error!("failed to create client");
    std::process::exit(1);
  };
  {
    let mut data = client.data.write().await;
    data.insert::<Env>(Arc::new(env));
  }
  if let Err(why) = client.start().await {
    error!("client error: {:#?}", why);
    std::process::exit(1);
  }
}

fn init() -> Result<Env, ()> {
  {
    use simplelog::*;
    CombinedLogger::init(vec![
      TermLogger::new(
        LevelFilter::Debug,
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
    if env.channel_id == 0 {
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

  Ok(env)
}
