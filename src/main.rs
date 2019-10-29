extern crate actix_web;
extern crate actix_web_actors;
extern crate futures;
extern crate json;
extern crate serde;
extern crate twitter_stream;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use futures::sync::mpsc as fmpsc;
use std::{thread, time};
use twitter_stream::rt::{self, Stream};
use twitter_stream::Token;

mod dashboard;
mod setup;
mod stream_listener;
mod tracking;
mod twitter_messages;
mod ws_client;

pub enum UpdateMessage {
    NewTweet(String),
    NewMinute,
}

fn index(_req: HttpRequest) -> &'static str {
    "Would be the dashboard"
}

fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let res = ws::start(ws_client::WebSocketClient::new(), &r, stream);
    res
}

fn main() {
    let (credentials, configuration) = setup::setup();

    let (req_tx, req_rx) = fmpsc::unbounded::<UpdateMessage>();

    // Starting the service for keeping track of tweets
    let mut tracking = tracking::Tracking::new(&configuration.track_keys);
    let mut dashboard = dashboard::Dashboard::new();
    thread::spawn(move || {
        let resolver_loop = req_rx.for_each(move |msg| {
            match msg {
                UpdateMessage::NewTweet(key) => tracking.increment_count_by_key(&key),
                UpdateMessage::NewMinute => tracking.minute_complete(),
            }
            dashboard.update(&tracking);
            Ok(())
        });
        rt::run(resolver_loop);
    });

    // Starting the http server for the dashboard
    let http_port = configuration.http_port.unwrap_or("8080".to_string());
    thread::spawn(move || {
        HttpServer::new(|| {
            App::new()
                .service(web::resource("/index.html").to(|| "Would be the dashboard"))
                .service(web::resource("/").to(index))
                .service(web::resource("/ws/").route(web::get().to(ws_index)))
        })
        .bind(format!("127.0.0.1:{}", http_port))
        .expect("http server to start")
        .run()
    });

    // Starting a thread what triggers the tracking service every minute
    // to aggregate data
    let sender_clone = req_tx.clone();
    thread::spawn(move || loop {
        let one_minute = time::Duration::from_secs(60);
        thread::sleep(one_minute);
        sender_clone
            .unbounded_send(UpdateMessage::NewMinute)
            .expect("Tracking service");
    });

    // Connecting to twitter
    // Todo: Restart strategie on disconnect.
    let token = Token::new(
        credentials.api_key,
        credentials.api_secret,
        credentials.access_token,
        credentials.access_secret,
    );
    let future = stream_listener::create_listener(configuration.track_keys, token, req_tx);
    rt::run(future);
}
