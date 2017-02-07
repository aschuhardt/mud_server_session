//session.rs

pub mod configuration;
pub mod file_io;

mod request_cache;

use std::io::Read;
use std::str::FromStr;
use std::net;
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;

use time;
use uuid::{Uuid};
use mud_engine::Engine;

use super::configuration::Configuration;
use mud_request::{Request, RequestType};
use self::request_cache::RequestCache;

const CACHE_EMIT_INTERVAL: i64 = 60;

pub struct Session<'a> {
    config: &'a Configuration,
    pub quit: bool,
}

impl<'a> Session<'a> {
    pub fn new(conf: &'a Configuration) -> Session {
        Session {
            config: conf,
            quit: false,
        }
    }

    pub fn run(&self) {
        let (req_tx, req_rs) = mpsc::channel();
        let engine = Engine::new(self.config.request_validation_token.clone());
        self.init_listener(req_tx);
        while !self.quit {
            //handle cached incoming requests
            if let Ok(c) = req_rs.try_recv() {
                //process request cache...
                for (id, req) in &c.requests {
                    if self.config.debug_mode {
                        println!("Processing request: {} from client {}", id, req.client_id);
                    };
                    engine.process_request(req);
                }
            }
            //perform engine cycle
            engine.perform_tick();
            //receive outgoing responses from engine
            let responses = engine.get_responses();
            for res in &responses {
                let res_clone = res.clone();
                let debug_mode = self.config.debug_mode;
                //send responses on seperate threads
                thread::spawn(move || {
                    //serialize + send out response
                    let (msg, success) = res_clone.send();
                    if !success && debug_mode {
                        println!("{}", msg);
                    }
                });
            }
        }
    }



    fn init_listener(&self, tx: mpsc::Sender<RequestCache>) {
        let local_port = self.config.network_port;
        let cache_capacity = self.config.max_request_cache_count;
        let debug_mode = self.config.debug_mode;
        let type_hashes = Request::create_request_type_hashes(&self.config.request_validation_token);
        thread::spawn(move || {
            let (l_tx, l_rx) = mpsc::channel();
            //initialize loop that will wait for requests returned from listeners and add them to a
            //  request cache to be transmitted via master_tx
            Session::funnel_requests_into_cache(tx, l_rx, cache_capacity, debug_mode);

            //start listening on the configured port and start threads for each incoming request
            let ipv4 = net::Ipv4Addr::from_str("127.0.0.1").unwrap();
            let addr = net::SocketAddrV4::new(ipv4, local_port);
            if let Ok(listener) = net::TcpListener::bind(addr) {
                if debug_mode {
                    println!("Started listening on port {}...", local_port);
                }
                for stream in listener.incoming() {
                    if let Ok(mut s) = stream {
                        Session::init_listener_thread(l_tx.clone(),
                                                      &mut s,
                                                      type_hashes.clone(),
                                                      debug_mode);
                    }
                }
            } else {
                println!("Failed to bind to port {}!", local_port);
            }
        });
    }

    fn funnel_requests_into_cache(cache_tx: mpsc::Sender<RequestCache>, l_rx: mpsc::Receiver<Request>,
                                  cache_cap: usize, debug_mode: bool) {
        let mut req_cache = RequestCache::new(debug_mode, cache_cap);
        thread::spawn(move || {
            let mut emit_interval_start = time::PreciseTime::now();
            loop {
                //cache incoming requsts from listener threads
                if let Ok(req) = l_rx.try_recv() {
                    req_cache.add(req);
                }

                //here we are waiting a constant time interval before sending the request cache
                //  back to the main session thread.  We are doing this in order to give the cache
                //  enough time to build up some requests before sending them to the main thread,
                //  ensuring (hopefully) that the main session thread has got a healthy sizeable
                //  workload.
                //TODO: Consider coming up with a less-hokey way of ensuring that the cache gets utilized correctly.
                if emit_interval_start.to(time::PreciseTime::now()) >= time::Duration::microseconds(CACHE_EMIT_INTERVAL) {
                    emit_interval_start = time::PreciseTime::now();
                    if let Err(why) = cache_tx.send(req_cache.clone()) {
                        panic!("Failed to emit request cache to main session thread: {}", why);
                    } else {
                        req_cache.clear();
                    }
                }
            }
        });
    }

    fn init_listener_thread(tx: mpsc::Sender<Request>, stream: &mut net::TcpStream, type_hashes: HashMap<Uuid, RequestType>, debug_mode: bool) {
        let mut buffer = String::new();
        if let Err(why) = stream.read_to_string(&mut buffer) {
            println!("Malformed or invalid stream buffer encountered from peer: {}. Reason: {}",
                     stream.peer_addr().unwrap(), why);
        } else if let Some(req) = Request::new(buffer.clone(), type_hashes, stream) {
            if let Err(why) = tx.send(req) {
                println!("Unable to send request to caching thread: {}", why);
            } else if debug_mode {
                println!("Request received: {}", buffer.as_str());
            }
        }
    }
}
