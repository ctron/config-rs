use std::error::Error;

use config::{builder::AsyncState, AsyncSource, ConfigBuilder, ConfigError, FileFormat, Map};

use async_trait::async_trait;
use futures::{select, FutureExt};
use warp::Filter;

// Example below presents sample configuration server and client.
//
// Server serves simple configuration on HTTP endpoint.
// Client consumes it using custom HTTP AsyncSource built on top of reqwest.

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    select! {
        r = run_server().fuse() => r,
        r = run_client().fuse() => r
    }
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    let service = warp::path("configuration").map(|| r#"{ "value" : 123 }"#);

    println!("Running server on localhost:5001");

    warp::serve(service).bind(([127, 0, 0, 1], 5001)).await;

    Ok(())
}

async fn run_client() -> Result<(), Box<dyn Error>> {
    // Good enough for an example to allow server to start
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let config = ConfigBuilder::<AsyncState>::default()
        .add_async_source(HttpSource {
            uri: "http://localhost:5001/configuration".into(),
            format: FileFormat::Json,
        })
        .build()
        .await?;

    println!("Config value is {}", config.get::<String>("value")?);

    Ok(())
}

// Actual implementation of AsyncSource can be found below

#[derive(Debug)]
struct HttpSource {
    uri: String,
    format: FileFormat,
}

#[async_trait]
impl AsyncSource for HttpSource {
    async fn collect(&self) -> Result<Map<String, config::Value>, ConfigError> {
        reqwest::get(&self.uri)
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))? // error conversion is possible from custom AsyncSource impls
            .text()
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))
            .and_then(|text| {
                self.format
                    .parse(Some(&self.uri), &text)
                    .map_err(|e| ConfigError::Foreign(e))
            })
    }
}
