use std::collections::HashMap;

use extism_pdk::{http, Error, FromBytes, HttpRequest, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, FromBytes)]
#[encoding(Json)]
pub struct PosthogClientConfig {
    api_key: String,
    host: String,
}

pub struct PosthogClient {
    config: PosthogClientConfig,
    event_buffer: Vec<PosthogEvent>,
}

#[derive(Serialize, Deserialize, FromBytes)]
#[encoding(Json)]
pub struct PosthogEvent {
    event: String,
    properties: HashMap<String, Value>,
}

#[derive(Serialize)]
pub struct BatchBody {
    api_key: String,
    historical_migration: bool,
    batch: Vec<PosthogEvent>,
}

impl PosthogClient {
    pub fn new(config: PosthogClientConfig) -> Self {
        Self {
            config,
            event_buffer: Vec::new(),
        }
    }

    pub fn capture(&mut self, event: PosthogEvent) {
        // Add additional properties to the event
        self.event_buffer.push(event);
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        let mut events = vec![];
        for event in self.event_buffer.drain(..) {
            events.push(event);
        }
        let body = BatchBody {
            api_key: self.config.api_key.clone(),
            historical_migration: false,
            batch: events,
        };
        let body_string = serde_json::to_string(&body).expect("failed to serialize body");
        let req = HttpRequest::new(format!("{}/batch", self.config.host))
            .with_method("POST")
            .with_header("Content-Type", "application/json");
        let resp = http::request::<String>(&req, Some(body_string))?;
        println!("{:?}", resp.body());
        Ok(())
    }
}
