# Top Players Tracker

CLI tool designed to track the top football players in the Premier League's 23/24 season. It fetches data from the Sportradar Soccer API and outputs the top 10 players with the most goals and assists.

## Setup

### Prerequisites

1. Rust (https://www.rust-lang.org/tools/install)
2. Sportradar API key (https://console.sportradar.com/signup)

### Configuration

After cloning this repository, you should create a `.env` file. You can copy the example found in the root folder:

```bash
$ cp .env.example .env
```

Then change the `SPORTRADAR_API_KEY` value to the one found in the Sportradar console web page.

## Usage

To compile and run you can use `cargo`, just like any other Rust project.

To build:

```bash
$ cargo build # debug
# or
$ cargo build --release
```

And to run (can used be directly without `build`):

```bash
$ cargo run # debug
# or
$ cargo run --release
```

There are four available commands:

- `top-assists`: Prints the top 10 players ordered by assists
- `top-goals`: Prints the top 10 players ordered by goals scored
- `top-players`: Prints the top 10 players ordered first by goals then assists
- `clear-cache`: Clears the cache files for the season data

Example output:

```bash
$ cargo run top-players 
fetching season data...
Goals | Assists | Player Name
 27 | 5 | Haaland, Erling
 22 | 11 | Palmer, Cole
 21 | 2 | Isak, Alexander
 19 | 13 | Watkins, Ollie
 19 | 8 | Foden, Phil
 19 | 3 | Solanke, Dominic
 18 | 10 | Salah, Mohamed
 17 | 10 | Heung-min, Son
 16 | 9 | Saka, Bukayo
 16 | 6 | Bowen, Jarrod
```

## Tests

As simple as the other commands, you can use:

```bash
$ cargo test
```

## Code Structure

- `main`: Entry point for the application
- `cmd`: Defines the command-line interface and available commands
- `api_client`: Contains the logic for interacting with the Sportradar API
- `cached_client`: Implements caching to minimize API calls
- `client`: Defines the `Client` trait used for fetching data
- `top_players`: Contains logic for processing and sorting player statistics
- `types`: Type definitions for the API structures

## Improvements

I ended up getting too excited about the code challenge, so I definitely spent more than a few hours doing it. I decided to stop at this current point because I think it shows a little bit of my code in various areas.

If I were to spend more time on this project, here are a few things that could be done to improve it:

- Parallelize API calls. This was not done because:
  - I faced a "Too Many Requests" error by doing more than a single request in a small time frame. This is probably because my API key was for the trial version. Perhaps the API could behave differently if a production key had been used.
  - The parallel version can be easily be done with something like `tokio::join!`.
  - And the cache that's being mutated as results come from the API, [dashmap](https://github.com/xacrimon/dashmap) could be used in the place of `HashMap`.
- Options for the number of players via CLI arguments
- Better printing. It could be done manually or by using a library.
- Parameterize the season. A request could be done to https://developer.sportradar.com/soccer/reference/soccer-competition-seasons prior to the other ones
- Show the player's team alongside their other data.
  - It could be done by changing a few function call signatures and storing the `String` alongside the `Player`
  - Or there could be a new type for player that is not a 1 to 1 match with the API so that it's easier to manipulate (I prefer this option).
- The file system calls could be abstracted to test the cache.
  - I decided to not do this because it would take a lot of time, and ultimately the cache could be anything, for example a Redis server, so the test may be thrown away
- In depth tests of the cache, I decided to do only a couple of base cases, since I was concerned with time.
- Integration and/or end to end tests.
- Documentation in code
  - Because the problem statement is simple, the code ended up being straight forward to read. Perhaps the most "complex" part is the cache
  - And since I was doing a `README.md` file alongside with it, I decided to give a high-level overview here
  - If the problem being solved is harder, more complex, or has many more layers, code documentation can be key to readability
  - Doc tests could be used too to show examples of how to use structures/functions
