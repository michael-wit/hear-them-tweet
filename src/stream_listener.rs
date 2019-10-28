use crate::twitter_messages::StreamMessage;
use crate::UpdateMessage;
use futures::sync::mpsc as fmpsc;
use twitter_stream::rt::{self, Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

pub fn create_listener(
    track_keys: Vec<String>,
    token: Token,
    key_sender: fmpsc::UnboundedSender<UpdateMessage>,
) -> impl Future<Item = (), Error = ()> {
    TwitterStreamBuilder::filter(token)
        .track(Some(track_keys.join(",").as_str()))
        .stall_warnings(true)
        .listen()
        .expect("Valid Autorization")
        .flatten_stream()
        .for_each(move |json| {
            match json::from_str(&json) {
                Ok(message) => match message {
                    StreamMessage::Tweet(tweet) => {
                        for key in track_keys.iter() {
                            // Right now just scanning the full json for the keyword.
                            // Twitter is not very specific how they tag the tweets.
                            // Gives a couple of false postitive specificly on 'art' and 'twitter'.
                            // Can be improved by searching for 'full wowrd' and a smaller
                            // amount of fields.
                            if json.contains(key) {
                                println!("{} lang: {}, id: {}", key, tweet.lang, tweet.id_str);
                                key_sender
                                    .unbounded_send(UpdateMessage::NewTweet(key.to_owned()))
                                    .expect("tracking service alive");
                            }
                        }
                    }
                    StreamMessage::Warning(warning) => {
                        println!(
                            "{}: {} full",
                            warning.warning.code, warning.warning.percent_full
                        );
                    }
                    StreamMessage::Limit(limit) => {
                        println!("Limit Track {}", limit.limit.track);
                    }
                    _ => {
                        println!("other message: {}", json);
                    }
                },
                Err(e) => {
                    println!("unparsed msg: {:?}", e);
                    println!("{}", json);
                }
            }
            Ok(())
        })
        .map_err(|e| println!("error: {}", e))
}
