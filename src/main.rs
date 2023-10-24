use models::camera::Camera;
use reqwest::Url;
use services::{discovery::DiscoveryService, onvif::RTSPUrl};

use crate::services::onvif::OnvifService; 

pub mod services;
pub mod models;


pub async fn run(rtsp_uri: &RTSPUrl, cam: &mut Camera) {
    let uri = Url::parse(rtsp_uri.uri.as_str()).ok().unwrap();

    let session = retina::client::Session::describe(
        uri,
        retina::client::SessionOptions::default()
            .creds(cam.get_rtsp_credentials())
            .user_agent("Retina sdp example".to_owned()),
    )
    .await.unwrap();

    println!("SDP:\n{}\n\n", std::str::from_utf8(session.sdp()).unwrap());

    for (i, stream) in session.streams().iter().enumerate() {
        println!("stream {i}:\n{stream:#?}\n\n");
    }

}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut discover = DiscoveryService::default();
    let mut cams = discover.discover().await.unwrap();

    for cam in &mut cams {
        cam.set_credentials(Some("admin".to_string()), Some("L24515F6".to_string()));

        let mut onv = OnvifService::default();
        onv.build_services(&cam).await;
        let urls = onv.get_stream_uris().await;

        for url in urls {
            run(&url, cam).await;
        }

    }

    println!("Ok!");

}
