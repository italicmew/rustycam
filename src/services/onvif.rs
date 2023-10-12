use std::collections::HashMap;

use onvif::soap::{self, client::ClientBuilder};
use reqwest::Url;

use crate::models::camera::Camera;


// devicemgmt: soap::client::Client,
// event: Option<soap::client::Client>,
// deviceio: Option<soap::client::Client>,
// media: Option<soap::client::Client>,
// media2: Option<soap::client::Client>,
// imaging: Option<soap::client::Client>,
// ptz: Option<soap::client::Client>,
// analytics: Option<soap::client::Client>,

pub struct OnvifService {
    onvi_clients: HashMap<String, Option<soap::client::Client>>
}


impl Default for OnvifService {
    fn default() -> Self {
        Self {
            onvi_clients: HashMap::new()
        }
    }
}

impl OnvifService {
    async fn build_services(&mut self, cam: &Camera) {
        let uri = cam.get_onvif_url().expect("Device onvif URL is expected.");
        let dvmg = ClientBuilder::new(&uri)
            .credentials(cam.get_onvif_credentials())
            .build();

        let services =
            schema::devicemgmt::get_services(&dvmg, &Default::default()).await.unwrap();
        
        for service in &services.service {
            println!(">>> {:#?}", service)
        }

    }
}