use std::io::ErrorKind;

use chrono::{FixedOffset, NaiveDate, Utc};
use log::{error, info};
use serde::Deserialize;
use serenity::all::{
  Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
  CreateInteractionResponseMessage,
};

use crate::{Env, Event};

pub fn register() -> CreateCommand {
  CreateCommand::new("show").description("Show upcoming events")
}

pub async fn execute(env: &Env, ctx: &Context, interaction: &CommandInteraction) {
  let Ok(embed) = build_embed() else {
    return;
  };
  match interaction
    .create_response(
      &ctx.http,
      CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
          .content("I remind you of upcoming events!")
          .embed(embed),
      ),
    )
    .await
  {
    Ok(_) => (),
    Err(e) => error!("{}", e),
  };
}

fn build_embed() -> Result<CreateEmbed, ()> {
  let today = Utc::now()
    .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
    .date_naive();
  #[derive(Deserialize)]
  struct EventsFile {
    events: Vec<Event>,
  }
  let events: EventsFile = match &std::fs::read_to_string("events.toml") {
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

  Ok(
    CreateEmbed::new()
      .title("Events")
      .color(Colour::new(0x98c379))
      .fields(events.events.iter().flat_map(|event| {
        let Ok(event_date) = NaiveDate::parse_from_str(&event.date, "%F") else {
          error!("Failed to parse event date: {:?}", event);
          return None;
        };
        if event_date < today {
          info!("Overdue event: {:?}", event);
          return None;
        }
        let days = (event_date - today).num_days();

        Some((
          event.title.clone(),
          format!("Due: {} day{}", days, if days == 1 { "" } else { "s" }),
          false,
        ))
      })),
  )
}
