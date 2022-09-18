use std::{error};
use log::{error, info};
use url::{Url};



/// Interpets the `username` and `password` of a [Source].
fn creds(
    username: Option<String>,
    password: Option<String>,
) -> Option<retina::client::Credentials> {
    match (username, password) {
        (Some(username), password) => Some(retina::client::Credentials {
            username,
            password: password.unwrap_or_default(),
        }),
        (None, None) => None,
        _ => unreachable!(), // structopt/clap enforce that password requires username.
    }
}


#[tokio::main]
async fn main() -> Result<(), retina::Error>{
    println!("Hello, world!");
    let url = Url::parse("rtsp://192.168.0.110/cam/realmonitor?channel=1&subtype=00&authbasic=YWRtaW46anBqNXJoa2E=").unwrap();
    let creds = creds(Some("admin".to_owned()), Some("".to_owned()));

    let session = retina::client::Session::describe(
        url,
        retina::client::SessionOptions::default().creds(creds).user_agent("test".to_owned())
    ).await.unwrap();
    for (i, stream) in session.streams().iter().enumerate() {
        print!("{i} - {}", stream.media());
    }

    Ok(())
}
