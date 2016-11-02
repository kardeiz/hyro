#[macro_use]
extern crate hyro;
extern crate hyper;

extern crate num_cpus;

use hyper::server::{Server, Request, Response};
use hyper::method::Method;

use std::io::Read;

fn handle_numbers(_: &Request, res: Response, id: i32) {
    res.send(format!("NUMBER: {}", id).as_bytes()).unwrap();
}

fn main() {

    let server = {        
        use std::time::Duration;

        let host = ::std::env::var("WEB_HOST")
            .unwrap_or("0.0.0.0".into());
        let port = ::std::env::var("WEB_PORT")
            .ok()
            .as_ref()
            .and_then(|x| x.parse().ok() )
            .unwrap_or(3000u16);

        let mut server = Server::http((&host as &str, port)).unwrap();
        server.keep_alive(Some(Duration::from_secs(5)));
        server.set_read_timeout(Some(Duration::from_secs(30)));
        server.set_write_timeout(Some(Duration::from_secs(1)));
        server
    };

    server.handle_threads(|mut req: Request, mut res: Response| {
        
        let matcher = hyro::Matcher::build(&req.uri);

        if let Some(m) = matcher.chomp(&"/foo") {
            if let Some(_) = m.complete("/").or(m.complete("")) {
                res.send(b"YES").unwrap();
                return;
            }
        }

        if let Some(m) = matcher.chomp('/')
            .and_then(|m| m.take_while(char::is_numeric)) {
            let (digits,) = m.captures();
            if digits.len() > 5 {
                res.send(b"that's a big number!").unwrap();
            } else {
                res.send(b"not a big number").unwrap();
            }
            return;
        }

        if let Some(m) = matcher.chomp("/bars/").iter()
            .flat_map(|m| m.take_while(char::is_numeric) )
            .flat_map(|m| m.chomp('/') )
            .map(|m| m.take_until('.') )
            .map(|m| m.take_rest() )
            .next() {
            
            let (id, action, format) = m.captures();

            res.send(format!("id: {}, action: {}, format: {}", id, action, format).as_bytes()).unwrap();

            return;
        }


        *res.status_mut() = hyper::status::StatusCode::NotFound;
        res.send(b"Path not found").unwrap();

    }, 8 * ::num_cpus::get()).unwrap();


}