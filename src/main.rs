// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
use serde::{Deserialize, Serialize};
use serenity::{
  all::{Command, Context, EventHandler, GatewayIntents, Interaction, Ready},
  async_trait,
  prelude::TypeMapKey,
  Client,
};
use std::{io::ErrorKind, path::Path, sync::Arc};
mod add;
mod events;
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

struct Handler;
#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, ready: Ready) {
    Command::set_global_commands(&ctx.http, vec![show::register(), add::register()])
      .await
      .unwrap();
    let ctx = ctx.clone();
    tokio::spawn(async move {
      let data = ctx.data.read().await;
      let Some(env) = data.get::<Env>() else {
        return;
      };
      show::notify_on_specified_time(&env, &ctx).await.unwrap();
    });
    println!(
      "INFO : ready> name: {}, version: {}",
      ready.user.name, ready.version
    );
  }
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    let Interaction::Command(command) = interaction else {
      return;
    };
    println!("INFO : command came: {:?}", command.data);
    match command.data.name.as_str() {
      "show" => show::execute(&ctx, &command).await,
      "add" => add::execute(&ctx, &command.data.options(), &command).await,
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
    println!("ERROR: failed to create client");
    std::process::exit(1);
  };
  {
    let mut data = client.data.write().await;
    data.insert::<Env>(Arc::new(env));
  }
  if let Err(why) = client.start().await {
    println!("ERROR: client error: {:#?}", why);
    std::process::exit(1);
  }
}

fn init() -> Result<Env, ()> {
  let env: Env = match &std::fs::read_to_string("env.toml") {
    Ok(s) => toml::from_str(s).unwrap(),
    Err(e) => {
      if e.kind() == ErrorKind::NotFound {
        println!(
          "ERROR: `env.toml` is not found. Try `cp sample-env.toml env.toml`\n({})",
          e
        );
      } else {
        println!("ERROR: {}", e);
      }
      return Err(());
    }
  };
  println!("debug: {}", toml::to_string(&env).unwrap());
  {
    if env.discord_bot_token.is_empty() {
      println!("ERROR: `settings.discord_bot_token` is empty");
      return Err(());
    }
    if env.channel_id == 0 {
      println!("ERROR: `settings.channel_id` is empty");
      return Err(());
    }
  }

  // initialize `events.toml` if it doesn't exist
  if !Path::new("events.toml").exists() {
    events::write(String::from(
      r#"# [[events]]
# title = "EventTitle"
# date = "YYYY-MM-DD" # ISO 8601
"#,
    ))
    .unwrap();
  }

  Ok(env)
}
