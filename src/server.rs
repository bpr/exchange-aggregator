use async_trait::async_trait;
use core::future::Future;
use ezsockets::ClientConfig;
use ezsockets::Error;
use futures::try_join;
use std::collections::HashMap;
use std::io::BufRead;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use url::Url;

use exchange_aggregator::orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use exchange_aggregator::orderbook::{ Empty, Summary };

mod data;

type State = std::sync::Arc<tokio::sync::RwLock<HashMap<String, Summary>>>;

// gRPC structs
#[derive(Debug)]
pub struct OrderbookService {
    state: State,
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookService {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;    

    async fn book_summary(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let state = self.state.clone();
        tokio::spawn(async move {
            let table = state.read().await;
            for summary in table.values() {
                println!("  => send {:?}", summary);
                tx.send(Ok(summary.clone())).await.unwrap();
            }

            println!(" /// done sending");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

// WebSocket structs

struct WSClient {
    name: String,
    init_string: Option<String>,
    handle: ezsockets::Client<Self>,
    state: State,
}

impl WSClient {
    fn new(name: String, handle: ezsockets::Client<Self>, init_string: Option<String>, state: State) -> Self {
        Self {
            name,
            init_string,
            handle,
            state,
        }
    }
}

#[async_trait]
impl ezsockets::ClientExt for WSClient {
    type Call = ();

    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        let name = self.name.clone();
        let mut table = self.state.write().await;
        table.insert(name.clone(), data::summary_of_string(text, &name));
        let value = table.get(&name).unwrap();
        tracing::info!("{} received message: {:?}", name, value);
        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        tracing::info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        let () = call;
        Ok(())
    }

    /// Called when the client successfully connected(or reconnected).
    async fn on_connect(&mut self) -> Result<(), Error> {
        if let Some(data) = self.init_string.take() {
            let _ = self.handle.text(data);
        };
        tracing::info!("connected!");
        Ok(())
    }
}

const BITSTAMP_WS_URL: &str = "wss://ws.bitstamp.net";
const BINANCE_WS_URL: &str = "wss://stream.binance.us:9443/ws/ethbtc@depth20@100ms";

async fn connect_to_url<'a>(
    init_params: Option<String>,
    url_str: &'a str,
    client_name: &'a str,
    state: State,
) -> (ezsockets::Client<WSClient>, impl Future<Output = Result<(), Error>> + 'a) {
    let url = Url::parse(url_str).unwrap();
    let config = ClientConfig::new(url);
    ezsockets::connect(|handle| WSClient::new(client_name.to_string(), handle, init_params, state), config).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:10000".parse().unwrap();

    println!("OrderbookServer listening on: {}", addr);

    let state = State::default();

    let route_guide = OrderbookService {
        state: state.clone(),
    };

    let svc = OrderbookAggregatorServer::new(route_guide);

    let bts_params = r#"
    {
        "event": "bts:subscribe",
        "data": {
            "channel": "order_book_ethbtc"
        }
    }"#;

    let bts_init = Some(bts_params.to_string());
    let (bts_handle, bts_future) =
        connect_to_url(bts_init, BITSTAMP_WS_URL, "bts", state.clone()).await;

    let bnb_init = None;
    let (bnb_handle, bnb_future) =
        connect_to_url(bnb_init, BINANCE_WS_URL, "bnb", state.clone()).await;

    let handles = vec![bts_handle, bnb_handle];
    for handle in handles {
        tokio::spawn(async move {
            let stdin = std::io::stdin();
            let lines = stdin.lock().lines();
            for line in lines {
                let line = line.unwrap();
                let _ = handle.text(line);
            }
        });
    }

    let joined_futures = try_join!(bts_future, bnb_future);
    joined_futures.unwrap();

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
