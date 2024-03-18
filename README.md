# Exchange Aggregator
A program that 
1. connects to two exchanges' websocket feeds at the same time,
2. pulls order books, using these streaming connections, for a given traded pair of currencies (configurable), from each exchange,
3. merges and sorts the order books to create a combined order book,
4. from the combined book, publishes the spread, top ten bids, and top ten asks, as a stream, through a gRPC server.

## Design
The program comprises two executables:
- A websocket client which is also a gRPC server. This client collects the     
  data from the two websocket servers, and stores each as an entry in a hashmap.
- A gRPC client which retrieves the data from the server, merges and sorts the 
  order books, and publishes the desired data.

## TODO
- Abstract away the differences between websocket clients
- Extend the implementation to handle more than two clients

## Running the program

Clone this repository:

``` sh
$ git clone https://www.github.com/bpr/exchange-aggregator
```

Run the server in a terminal window:
``` sh
$ cargo run --bin server
```

Run the client in another terminal window:
``` sh
$ cargo run --bin client
```
