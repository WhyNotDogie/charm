# Charm
A simple chat app made to demonstrate the rust `packetz` library.

## Usage
Running the server:
```sh
cargo run --bin server localhost:5515
```
Running the client:
```sh
cargo run --bin client localhost:5515
```

Here is some example output:
```
[SERVER]: 127.0.0.1:54154 just connected!
[SERVER]: 127.0.0.1:54166 just connected!
[127.0.0.1:54154]: hey!
[127.0.0.1:54166]: whats up?
[127.0.0.1:54154]: nothing much, you?
[127.0.0.1:54166]: this is a really cool chatting app. whoever made it is really cool.
```