use models::camera::Camera;
use services::discovery::DiscoveryService; 

pub mod services;
pub mod models;


async fn get_hostname(cam: &Camera) {
    // let creds: Option<Credentials> = Some(Credentials {
    //     password: cam,
    //     username: cam.user.clone()
    // });
    // let dvmg = ClientBuilder::new(&cam.device.urls[0])
    //     .credentials(creds.clone())
    //     .build();

    // println!("omg");
    // let resp = devicemgmt::get_hostname(&dvmg, &Default::default()).await.unwrap();

    // let resp2 = schema::media::get_profiles(&dvmg, &Default::default()).await;

    // let profiles = resp2.unwrap().profiles;

    // println!(
    //     "{}",
    //     resp.hostname_information
    //         .name
    //         .as_deref()
    //         .unwrap_or("(unset)")
    // );

}



#[tokio::main]
async fn main() {
    let mut discover = DiscoveryService::default();
    let a = discover.discover().await.unwrap();

    println!("Ok!");

}
