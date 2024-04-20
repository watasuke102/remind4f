use chrono::NaiveDate;
use serenity::all::{
  CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
  CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue,
};

use crate::events::{self, EventFileEntry, EventsFile};

pub fn register() -> CreateCommand {
  CreateCommand::new("add")
    .description("Add new events")
    .add_option(
      CreateCommandOption::new(CommandOptionType::String, "title", "Event title").required(true),
    )
    .add_option(
      CreateCommandOption::new(CommandOptionType::String, "date", "Event due date").required(true),
    )
}

pub async fn execute(
  ctx: &Context,
  options: &[ResolvedOption<'_>],
  interaction: &CommandInteraction,
) {
  let content = (|| {
    let ResolvedOption {
      value: ResolvedValue::String(title),
      ..
    } = options[0]
    else {
      return String::from("Invalid argument: 'title'");
    };
    let ResolvedOption {
      value: ResolvedValue::String(date),
      ..
    } = options[1]
    else {
      return String::from("Invalid argument: 'date'");
    };
    let Ok(date_naive) = NaiveDate::parse_from_str(&date, "%F") else {
      return format!("Failed to parse date; please enter ISO 8601-formatted date (YYYY-MM-DD)",);
    };
    let Ok(mut events) = events::read_events() else {
      return String::from("Failed to read events");
    };
    events.push(crate::events::Event {
      title: title.to_string(),
      date:  date_naive,
    });
    events.sort_by(|a, b| a.date.cmp(&b.date));
    let events_file = EventsFile {
      events: events
        .into_iter()
        .map(|e| EventFileEntry {
          title: e.title,
          date:  e.date.to_string(),
        })
        .collect(),
    };
    String::from(match events::write(events_file.to_string()) {
      Ok(_) => "Created!",
      Err(_) => "Failed to write events to file",
    })
  })();
  if let Err(e) = interaction
    .create_response(
      &ctx.http,
      CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(content)),
    )
    .await
  {
    println!("ERROR: cannot create response (add) => {}", e);
  };
}
