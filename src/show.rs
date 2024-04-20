use chrono::{FixedOffset, NaiveTime, Timelike, Utc};
use serenity::all::{
  ChannelId, Colour, CommandInteraction, Context, CreateCommand, CreateEmbed,
  CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
};

use crate::Env;

pub fn register() -> CreateCommand {
  CreateCommand::new("show").description("Show upcoming events")
}

pub async fn execute(ctx: &Context, interaction: &CommandInteraction) {
  let Ok(embed) = build_embed() else {
    return;
  };
  if let Err(e) = interaction
    .create_response(
      &ctx.http,
      CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
          .content("Upcoming events are following:")
          .embed(embed),
      ),
    )
    .await
  {
    println!("ERROR: {}", e);
  }
}

pub async fn notify_on_specified_time(env: &Env, ctx: &Context) -> Result<(), ()> {
  let jst = FixedOffset::east_opt(9 * 3600).unwrap();
  let Ok(notice_time) = NaiveTime::parse_from_str(&env.notice_time, "%H:%M") else {
    println!("ERROR: Failed to parse `notice_time`; please check data.toml");
    return Err(());
  };

  println!("INFO : Bot is ready");
  loop {
    println!("debug: tick");
    let now = Utc::now().with_timezone(&jst);
    if now.time().hour() == notice_time.hour() && now.time().minute() == notice_time.minute() {
      println!("INFO : On time!");
      if let Ok(embed) = build_embed() {
        if let Err(e) = ChannelId::new(env.channel_id)
          .send_message(
            &ctx.http,
            CreateMessage::new()
              .content(format!(
                "{}I remind you of upcoming events!",
                if env.disable_everyone {
                  ""
                } else {
                  "@everyone "
                },
              ))
              .embed(embed),
          )
          .await
        {
          println!("ERROR: Failed to send regular message: {}", e);
        }
      }
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(1000 * 60)).await;
  }
}

fn build_embed() -> Result<CreateEmbed, ()> {
  let Ok(events) = crate::events::read_events() else {
    return Err(());
  };
  let today = Utc::now()
    .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
    .date_naive();
  let fields: Vec<(String, String, bool)> = events
    .iter()
    .flat_map(|event| {
      if event.date < today {
        println!("INFO : Overdue event: {:?}", event);
        return None;
      }
      let days = (event.date - today).num_days();

      Some((
        event.title.clone(),
        format!(
          "Date: {}\nDue: __**{} day{}**__",
          event.date,
          days,
          if days == 1 { "" } else { "s" }
        ),
        false,
      ))
    })
    .collect();
  if fields.is_empty() {
    return Err(());
  }

  Ok(
    CreateEmbed::new()
      .title("Events")
      .color(Colour::new(0x98c379))
      .fields(fields),
  )
}
