use serde::{Deserialize, Serialize};
use exchange_aggregator::orderbook::{Level, Summary};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketData {
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}

impl Default for MarketData {
    fn default() -> Self {
        MarketData {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BtsMarketData {
    bids: Option<Vec<(String, String)>>,
    asks: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WrappedBtsMarketData {
    data: BtsMarketData,
}

fn market_data_from_string(data: String, exchange_name: &str) -> MarketData {
    if exchange_name == "bts" {
        let wrapped = serde_json::from_str::<WrappedBtsMarketData>(&data)
            .expect(&format!("{}: failed to deserialize {}", exchange_name, &data));
        MarketData {
            bids: wrapped.data.bids.unwrap_or_default(),
            asks: wrapped.data.asks.unwrap_or_default(),
        }
    } else {
        serde_json::from_str::<MarketData>(&data)
            .expect(&format!("{}: failed to deserialize {}", exchange_name, &data))
    }
}

pub fn summary_of_string(data: String, exchange_name: &str) -> Summary {
    let market_data = market_data_from_string(data, exchange_name);
    let mut bids_levels = market_data.bids.iter()
        .map(|(x, y)| Level { exchange: exchange_name.to_string(), price: x.parse::<f64>().unwrap(), amount: y.parse::<f64>().unwrap() })
        .collect::<Vec<exchange_aggregator::orderbook::Level>>();
    let mut asks_levels = market_data.bids.iter()
        .map(|(x, y)| Level { exchange: exchange_name.to_string(), price: x.parse::<f64>().unwrap(), amount: y.parse::<f64>().unwrap() })
        .collect::<Vec<Level>>();
    bids_levels.sort_by(|a, b| b.price.total_cmp(&a.price)); // decreasing order
    asks_levels.sort_by(|a, b| a.price.total_cmp(&b.price)); // increasing order
    let asks_levels_price = if asks_levels.len() > 0 { asks_levels[0].price } else { 0.0 };
    let bids_levels_price = if bids_levels.len() > 0 { bids_levels[0].price } else { 0.0 };
    Summary {
        spread: asks_levels_price - bids_levels_price,
        bids: bids_levels,
        asks: asks_levels,
    }
}

pub fn summary_of_file(data_dir: std::path::PathBuf, exchange_name: &str) -> Summary {
    let exchange_file =
        std::fs::read_to_string(data_dir.join(&format!("{}.json", exchange_name)))
        .expect(&format!("failed to open {} file", exchange_name));
    summary_of_string(exchange_file, exchange_name)
}

pub fn merge_summaries(summaries: Vec<Summary>) -> Summary {
    let mut bids_levels = Vec::new();
    let mut asks_levels = Vec::new();
    for summary in summaries {
        bids_levels.extend(summary.bids);
        asks_levels.extend(summary.asks);
    }
    bids_levels.sort_by(|a, b| b.price.total_cmp(&a.price)); // decreasing order
    asks_levels.sort_by(|a, b| a.price.total_cmp(&b.price)); // increasing order
    let asks_levels_price = if asks_levels.len() > 0 { asks_levels[0].price } else { 0.0 };
    let bids_levels_price = if bids_levels.len() > 0 { bids_levels[0].price } else { 0.0 };
    Summary {
        spread: asks_levels_price - bids_levels_price,
        bids: bids_levels,
        asks: asks_levels,
    }
}

#[allow(dead_code)]
pub fn load() -> Vec<exchange_aggregator::orderbook::Summary> {
    let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data"]);
    let binance_summary = summary_of_file(data_dir.clone(), "binance");
    let bitstamp_summary = summary_of_file(data_dir.clone(), "bitstamp");

    vec![merge_summaries(vec![binance_summary, bitstamp_summary])]
}

/*

    let binance_string = std::fs::read_to_string(data_dir.join("binance.json")).expect("failed to open binance file");
    let bitstamp_string = std::fs::read_to_string(data_dir.join("bitstamp.json")).expect("failed to open bitstamp file");

    let binance_decoded: MarketData = serde_json::from_str::<MarketData>(&binance_string).expect("failed to deserialize binance_features");
    let bitstamp_decoded: MarketData = serde_json::from_str::<MarketData>(&bitstamp_string).expect("failed to deserialize binance_features");

    let binance_bids_levels = binance_decoded.bids.iter()
        .map(|(x, y)| crate::orderbook::Level { exchange: "binance".to_string(), price: x.parse::<f64>().unwrap(), amount: y.parse::<f64>().unwrap() })
        .collect::<Vec<crate::orderbook::Level>>();     
    let binance_asks_levels = binance_decoded.asks.iter()
        .map(|(x, y)| crate::orderbook::Level { exchange: "binance".to_string(), price: x.parse::<f64>().unwrap(), amount: y.parse::<f64>().unwrap() })
        .collect::<Vec<crate::orderbook::Level>>();     
    let bitstamp_bids_levels = bitstamp_decoded.bids.iter() 
        .map(|(x, y)| crate::orderbook::Level { exchange: "bitstamp".to_string(), price: x.parse::<f64>().unwrap(), amount: y.parse::<f64>().unwrap() })
        .collect::<Vec<crate::orderbook::Level>>();
    let bitstamp_asks_levels = bitstamp_decoded.asks.iter() 
        .map(|(x, y)| crate::orderbook::Level { exchange: "bitstamp".to_string(), price: x.parse::<f64>().unwrap(), amount: y.parse::<f64>().unwrap() })
        .collect::<Vec<crate::orderbook::Level>>();
 */
