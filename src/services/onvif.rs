use std::{collections::HashMap, error::Error, fmt};

use log::{debug, info};
use onvif::soap::{self, client::ClientBuilder};
use reqwest::Url;

use crate::models::camera::Camera;

pub struct VideoResolution {
    pub width: i32,
    pub height: i32,
}

pub struct RTSPUrl {
    name: String,
    pub uri: String,
    resolution: VideoResolution
}

impl fmt::Display for VideoResolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl fmt::Display for RTSPUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}, URI: {}, Resolution: {}",
            self.name, self.uri, self.resolution
        )
    }
}

pub struct OnvifService {
    onvi_clients: HashMap<String, Option<soap::client::Client>>,
}

impl Default for OnvifService {
    fn default() -> Self {
        Self {
            onvi_clients: HashMap::new(),
        }
    }
}

impl OnvifService {
    pub async fn build_services(&mut self, cam: &Camera) {
        let uri = cam.get_onvif_url().expect("Device onvif URL is expected.");
        let dvmg = ClientBuilder::new(&uri)
            .credentials(cam.get_onvif_credentials())
            .build();

        let services = schema::devicemgmt::get_services(&dvmg, &Default::default())
            .await
            .unwrap();

        for service in &services.service {
            let svc_type = match service.namespace.as_str() {
                "http://www.onvif.org/ver10/events/wsdl" => Some("events"),
                "http://www.onvif.org/ver10/device/wsdl" => Some("device"),
                "http://www.onvif.org/ver10/deviceIO/wsdl" => Some("deviceIO"),
                "http://www.onvif.org/ver10/media/wsdl" => Some("media"),
                "http://www.onvif.org/ver20/media/wsdl" => Some("media2"),
                "http://www.onvif.org/ver20/imaging/wsdl" => Some("imaging"),
                "http://www.onvif.org/ver20/ptz/wsdl" => Some("ptz"),
                "http://www.onvif.org/ver20/analytics/wsdl" => Some("analytics"),
                _ => {
                    println!("{}", service.namespace);
                    None
                }
            };

            if let Some(value) = svc_type {
                let service_url = Url::parse(&service.x_addr).ok();
                let svc = match service_url {
                    Some(url) => Some(
                        soap::client::ClientBuilder::new(&url)
                            .credentials(cam.get_onvif_credentials())
                            .build(),
                    ),
                    None => None,
                };
                self.onvi_clients.insert(value.to_string(), svc);
            } else {
                println!("Not a valid service.");
            }
        }
    }

    pub async fn get_stream_uris(&mut self) -> Vec<RTSPUrl>{
        let media_client = self.access_service("media").unwrap();
        let profiles = schema::media::get_profiles(media_client, &Default::default()).await.unwrap();

        let requests: Vec<_> = profiles
        .profiles
        .iter()
        .map(|p: &schema::onvif::Profile| schema::media::GetStreamUri {
            profile_token: schema::onvif::ReferenceToken(p.token.0.clone()),
            stream_setup: schema::onvif::StreamSetup {
                stream: schema::onvif::StreamType::RtpUnicast,
                transport: schema::onvif::Transport {
                    protocol: schema::onvif::TransportProtocol::Rtsp,
                    tunnel: vec![],
                },
            },
        })
        .collect();

        let responses = futures_util::future::try_join_all(
            requests
                .iter()
                .map(|r| schema::media::get_stream_uri(media_client, r)),
        )
        .await.unwrap();

        info!("get_profiles response: {:#?}", &responses);

        let mut uris = Vec::new();

        for (p, resp) in profiles.profiles.iter().zip(responses.iter()) {
            if let Some(ref v) = p.video_encoder_configuration {
                let uri = RTSPUrl{
                    name: p.name.0.clone(),
                    uri: resp.media_uri.uri.clone(),
                    resolution: VideoResolution { width: (v.resolution.width), height: (v.resolution.height) }
                };
                uris.push(uri);
            }
        }

        uris
    
    }

    fn access_service(&self, service_key: &str) -> Option<&soap::client::Client> {
        self.onvi_clients
            .get(service_key)
            .and_then(|client| client.as_ref())
    }
}
