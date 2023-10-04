use std::io;
use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::{fs, collections::HashMap};

use serde_derive::Deserialize;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use xmltree::Element;
use std::process::exit;
extern crate socket2;

#[macro_use]
extern crate lazy_static;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};

pub const PORT: u16 = 3702;
lazy_static! {
    pub static ref IPV4: IpAddr = Ipv4Addr::new(239,255, 255, 250).into();
    pub static ref IPV6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123).into();
}


#[derive(Deserialize, Debug)]
struct Camera {
    name: String,
    ip: String,
    port: u16,
    user: String,
    password: String
}

fn join_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let ip_addr = addr.ip();

    let socket = new_socket(&addr)?;

    // depending on the IP protocol we have slightly different work
    match ip_addr {
        IpAddr::V4(ref mdns_v4) => {
            // join to the multicast address, with all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }
        IpAddr::V6(ref mdns_v6) => {
            // join to the multicast address, with all interfaces (ipv6 uses indexes not addresses)
            socket.join_multicast_v6(mdns_v6, 0)?;
            socket.set_only_v6(true)?;
        }
    };

    // bind us to the socket address.
    bind_multicast(&socket, &addr)?;

    // convert to standard sockets
    Ok(socket.into_udp_socket())
}

fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    socket.bind(&socket2::SockAddr::from(*addr))
}

fn multicast_listener(
    response: &'static str,
    client_done: Arc<AtomicBool>,
    addr: SocketAddr,
) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let server_barrier = Arc::new(Barrier::new(2));
    let client_barrier = Arc::clone(&server_barrier);

    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            // socket creation will go here...
            let listener = join_multicast(addr).expect("failed to create listener");
            println!("{}:server: joined: {}", response, addr);

            server_barrier.wait();
            println!("{}:server: is ready", response);

            // We'll be looping until the client indicates it is done.
            while !client_done.load(std::sync::atomic::Ordering::Relaxed) {
                // test receive and response code will go here...
                let mut buf = [0u8; 2048]; // receive buffer

                // we're assuming failures were timeouts, the client_done loop will stop us
                match listener.recv_from(&mut buf) {
                    Ok((len, remote_addr)) => {
                        let data = &buf[..len];

                        println!(
                            "{}:server: got data: {} from: {}",
                            response,
                            String::from_utf8_lossy(data),
                            remote_addr
                        );

                        // create a socket to send the response
                        let responder = new_socket(&remote_addr)
                            .expect("failed to create responder")
                            .into_udp_socket();

                        // we send the response that was set at the method beginning
                        responder
                            .send_to(response.as_bytes(), &remote_addr)
                            .expect("failed to respond");

                        println!("{}:server: sent response to: {}", response, remote_addr);
                    }
                    Err(err) => {
                        println!("{}:server: got an error: {}", response, err);
                    }
                }
            }

            println!("{}:server: client is done", response);
        })
        .unwrap();

    client_barrier.wait();
    join_handle
}



fn new_socket(addr: &SocketAddr) -> io::Result<Socket>{
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    Ok(socket)
}

fn new_sender(addr: &SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(addr)?;

    if addr.is_ipv4() {
        socket.set_multicast_if_v4(&Ipv4Addr::new(0, 0, 0, 0))?;

        socket.bind(&SockAddr::from(SocketAddr::new(
            Ipv4Addr::new(0, 0, 0, 0).into(),
            0,
        )))?;
    } else {
        // *WARNING* THIS IS SPECIFIC TO THE AUTHORS COMPUTER
        //   find the index of your IPv6 interface you'd like to test with.
        socket.set_multicast_if_v6(5)?;

        socket.bind(&SockAddr::from(SocketAddr::new(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
            0,
        )))?;
    }

    // convert to standard sockets...
    Ok(socket.into_udp_socket())
}


fn main() {
    let filename = "config/cameras.toml";

    let file_content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => todo!(),
    };

    let data: HashMap<String, Vec<Camera>> = match toml::from_str(&file_content) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Unable to load data from `{}`", err);
            exit(1);
        }
    };

    println!("Hello, world! {:?}", data);
}


/// This will guarantee we always tell the server to stop
struct NotifyServer(Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}


#[test]
fn test_ipv4_multicast() {
    test_multicast("ipv4", *IPV4);
}

#[test]
fn test_ipv6_multicast() {
    test_multicast("ipv6", *IPV6);
}


/// Our generic test over different IPs
fn test_multicast(test: &'static str, addr: IpAddr) {
    assert!(addr.is_multicast());
    let addr = SocketAddr::new(addr, PORT);

    let client_done = Arc::new(AtomicBool::new(false));
    let notify = NotifyServer(Arc::clone(&client_done));

    // multicast_listener(test, client_done, addr);

    // client test code send and receive code after here
    println!("{}:client: running", test);

    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?> 
        <e:Envelope xmlns:e="http://www.w3.org/2003/05/soap-envelope" xmlns:w="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:d="http://schemas.xmlsoap.org/ws/2005/04/discovery" xmlns:dn="http://www.onvif.org/ver10/network/wsdl">
        <e:Header> 
            <w:MessageID>uuid:4163f22f-2394-43e9-9635-dff01f9b7cc4</w:MessageID> 
            <w:To>urn:schemas-xmlsoap-org:ws:2005:04:discovery</w:To> 
            <w:Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</w:Action> 
        </e:Header> 
        <e:Body> 
            <d:Probe> 
                <d:Types>dn:NetworkVideoTransmitter</d:Types> 
            </d:Probe> 
        </e:Body> 
        </e:Envelope>"#
    );

    let mut names_element = Element::parse(message.as_bytes()).unwrap();

    println!("{:#?}", names_element);

    // create the sending socket
    let socket = new_sender(&addr).expect("could not create sender!");
    socket.send_to(message.as_bytes(), &addr).expect("could not send_to!");

    let mut buf = [0u8; 2048]; // receive buffer

    match socket.recv_from(&mut buf) {
        Ok((len, remote_addr)) => {
            let data = &buf[..len];
            let response = String::from_utf8_lossy(data);

            println!("{}:client: ---> got data: {}", test, response);

            // verify it's what we expected
            assert_eq!(test, response);
        }
        Err(err) => {
            println!("{}:client: had a problem: {}", test, err);
            assert!(false);
        }
    }

    // make sure we don't notify the server until the end of the client test
    drop(notify);
}
