use actix_web::{Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
pub enum Mode {
    Production,
    Presentation,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "production" => Ok(Mode::Production),
            "presentation" => Ok(Mode::Presentation),
            _ => Err(()),
        }
    }
}

impl FromRequest for Mode {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let mode = req.match_info().get("mode").unwrap().parse().unwrap();
        ready(Ok(mode))
    }
}
