extern crate mud_server_session;
extern crate uuid;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::net;
use mud_server_session::session::*;
use mud_server_session::session::remote::request::*;
use mud_server_session::configuration::*;
use uuid::Uuid;
use rustc_serialize::json;
use std::{thread, time};
use std::str::FromStr;

#[test]
fn test_request_receipt() {
    //create config
    let config = Configuration::load();
    let port = config.network_port;
    let validation_token = config.request_validation_token.as_str();

    //start a session to listen for requests
    thread::spawn(|| {
        let s_config = Configuration::load();
        let p_session = Session::new(&s_config);
        p_session.run();
    });

    //give server time to start
    let five_secs = time::Duration::from_millis(5000);
    thread::sleep(five_secs);

    //send a bunch of packets corresponding to the different requests types
    for i in 0..1024 {
        //get type hashes
        let type_hashes = Session::new(&config).create_request_type_hashes(validation_token);

        for (h, t) in type_hashes {
            //build request content payload
            let payload = "Hello world!".as_bytes().to_vec();

            //build request
            let req = SerializableRequest {
                client_id: Uuid::new_v4(),
                req_type: h,
                contents: payload,
            };

            //serialize request
            let serialized_req = json::encode(&req).unwrap();
            let req_bytes = serialized_req.as_bytes();

            let ipv4 = net::Ipv4Addr::from_str("127.0.0.1").unwrap();
            let addr = net::SocketAddrV4::new(ipv4, port);
            let mut stream = net::TcpStream::connect(addr).unwrap();
            let _ = stream.write(&req_bytes);
        }
    }

    //give server time to finish processing requests before
    thread::sleep(five_secs);
}
