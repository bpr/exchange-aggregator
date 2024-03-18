use std::error::Error;
use tonic::Request;
use tonic::transport::Channel;
use exchange_aggregator::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use exchange_aggregator::orderbook::Empty;

async fn print_summaries(client: &mut OrderbookAggregatorClient<Channel>) -> Result<(), Box<dyn Error>> {
    let request = Request::new(Empty {});
    let mut stream = client
        .book_summary(request)
        .await?
        .into_inner();

    while let Some(summary) = stream.message().await? {
        println!("NOTE = {:?}", summary);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:10000").await?;

    println!("\n*** SERVER STREAMING ***");
    print_summaries(&mut client).await?;

    Ok(())
}

