# alamo

Command-line tool that lists upcoming showtimes at the Alamo Drafthouse in
DTLA. It pulls the Alamo schedule feeds and writes a single static HTML page to
stdout.

## Usage

Print the page to a file:

```
cargo run > showtimes.html
```

## Install

Build a release binary:

```
cargo build --release
```

## Tests

```
cargo test
```
