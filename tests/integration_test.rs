extern crate mud_server_session;
extern crate mud_request;
extern crate uuid;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::net;
use std::{thread, time};
use std::str::FromStr;

use rustc_serialize::json;
use uuid::Uuid;

use mud_server_session::session::*;
use mud_request::request::*;
use mud_server_session::configuration::*;
use mud_server_session::file_io::*;

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
    for _ in 0..2 {
        //get type hashes
        let type_hashes = Request::create_request_type_hashes(validation_token);

        for (h, _) in type_hashes {
            //build request content payload
            let payload = b"Hello world!".to_vec();

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
            let _ = stream.write(req_bytes);
        }
    }

    //give server time to finish processing requests before
    thread::sleep(five_secs);
}

#[test]
fn test_config_io() {
    let test_config_path = "testconfig.json";
    let test_conf = Configuration {
        data_location: "data/data.json".to_string(),
        network_port: 10722,
        debug_mode: true,
        max_request_cache_count: 64,
        request_validation_token: "d5695226-2508-4187-b1eb-bed9665fbf26".to_string(),
    };
    //serialize + save configuration object
    Configuration::save_config_path(&test_conf, test_config_path);
    //deserialize + read configuration object
    let loaded_conf = Configuration::load_path(test_config_path);
    //test whether the loaded file has the test data in it
    assert_eq!(test_conf.data_location, loaded_conf.data_location);
    assert_eq!(test_conf.network_port, loaded_conf.network_port);
    //clean up test file
    FileIO::delete_file(test_config_path);
}

#[test]
fn test_file_io() {
    let test_path = "./testfile.txt";
    let test_contents = "asdf";
    //write some string to a file
    FileIO::write_string(test_path, test_contents);
    //read the file into a string
    let file_contents = FileIO::read_string(test_path);
    //test that the result of the read operation is what we expect
    assert_eq!(test_contents, file_contents);
    //clean up test file
    FileIO::delete_file(test_path);
}
