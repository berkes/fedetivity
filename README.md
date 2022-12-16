# Fedetivity

Automate your mastodon account.

## Component: Fedetivity

Is one component at the moment. This does Everything. But will probably
be split up in more microservices in future when the boundaries of these
become clear.

```
Websocket =[Stream]=> Transmitter =[Activity]=> Determinator =[Job]=>
Worker
```

TODO: make a proper UML for this

## Tools

TODO: write about tools

## Quickstart

TODO: write about cargo, rust, webmocket, mastodon auth etc.

### Install

TODO: introduce make install.


### Run

TODO: introduce make

### Test

After installing the dependencies, on the development machine, run

    cargo test

This builds and runs the tests locally.
Ensure a webmocket server is running.

### Release

TODO: introduce make release

### Deploy

TODO: introduce a CI to deploy and describe that here.

