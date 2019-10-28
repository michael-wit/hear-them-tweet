# hear-them-tweet

## Requirements

* Requires Rust (tested with 1.38.0)

* Requires Twitter API credentials

  Create a Json file with name 'credentials.json':
```Json
{
  "api_key": "<your_api_key>",
  "api_secret": "<your_api_secret>",
  "access_token": "<your_access_token>",
  "access_secret": "<your_access_secret>"
}
```

* The 'config.json' file can be edited to change the list of keywords or the http port.

## Run

```bash
cargo run
```

Should accept http and websocket requests at 'localhost:8080'

## Architecture

The project contains a few services (at the moment mainly thread, plan to move them all to Actors) loosely coupled communicating over message channels.

![Architecture](images/architecture.png)
