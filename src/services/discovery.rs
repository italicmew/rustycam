use std::time::Duration;

use futures_util::StreamExt;
use onvif::discovery::{self};

use crate::models::camera::Camera;


#[derive(Debug)]
pub struct DiscoveryService {
    duration: Duration
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(5)
        }
    }
}

impl DiscoveryService {
    // this is actually returning a maybe camera, we are not able to connect to it yet
    pub async fn discover(&mut self) -> Result<Vec<Camera>, onvif::discovery::Error>{
        let mut devices = match discovery::DiscoveryBuilder::default().duration(self.duration).run().await {
            Ok(d) => d,
            Err(error) =>  {
                eprintln!("Unable to load data from `{}`", error);
                return Err(error)
            }
        };
        
        let mut cameras = Vec::new();

        while let Some(device) = devices.next().await {
            let camera_config: Camera = device.into(); 
            cameras.push(camera_config);
        }
        
        Ok(cameras)
    }

}