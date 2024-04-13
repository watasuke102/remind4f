// remind4f - main.rs
//
// CopyRight (c) 2024 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT or MIT SUSHI-WARE License.
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
  let data: Data = match &std::fs::read_to_string("data.toml") {
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
  };
  println!("{:#?}", data);
}
