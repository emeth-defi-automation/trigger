use ethers::{
    contract::abigen,
    core::types::{Address, Filter},
    providers::{Provider, Ws},
};
use ethers::prelude::*;
use eyre::Result;
use serde_json::Number;
use std::{sync::Arc};
use tokio;
use chrono::{Utc};

abigen!(
    MyContract,
    r#"[
    event ActionExecutionAttempted(uint256 actionId, string message, uint256 timeZero, address contractAddress)
    function executeAction(uint256 actionId) public returns (bool)
    ]"#,
);


const WSS_URL: &str = "wss://eth-sepolia.g.alchemy.com/v2/573YG81S6_ZUn8IWf6IPz5YopOeanlaj";
const TOKEN_DELEGATOR_CONTRACT_ADDRESS: &str="0x48745dF521aC6a2822043F73d92b4045C9109246";

#[tokio::main]
async fn main() -> Result<()> {
    // let provider = Provider::<Ws>::connect(WSS_URL).await?;
    // let client = Arc::new(provider);

    // let now = Utc::now();
    // let timestamp = now.timestamp();

    // println!("Current Unix timestamp: {}", timestamp);

    // let latest_block = client.get_block_number().await?;

    // let target_address: Address = TOKEN_DELEGATOR_CONTRACT_ADDRESS.parse()?;

    // println!("{} block", latest_block);
    // println!("{} address", target_address);

    // let filter = Filter::new()
    // .address(target_address)
    // .event("ActionExecutionAttempted(uint256,string,uint256,address)")
    // .from_block(0);
    let provider = Provider::<Ws>::connect(WSS_URL).await?;
    let client = Arc::new(provider);
    let address: Address = TOKEN_DELEGATOR_CONTRACT_ADDRESS.parse()?;
    let contract = MyContract::new(address, client.clone());

    let latest_block = client.get_block_number().await?;
    println!("{} block", latest_block);

    listen_specific_events(&contract, &latest_block).await?;

    Ok(()) 
}


async fn listen_specific_events(contract: &MyContract<Provider<Ws>>,latest_block:&U64) -> Result<()> {
    
    let events = contract.event::<ActionExecutionAttemptedFilter>().from_block(latest_block);
    let mut stream = events.stream().await?.take(1);

    while let Some(Ok(event)) = stream.next().await {
        println!("SomeEvent event: {event:?}");
        println!("SomeEvent event: {:?}", event.message);
        let action_id = event.action_id;
        match contract.execute_action(action_id).send().await {
            Ok(tx) => println!("Transaction sent: {:?}", tx),
            Err(e) => println!("Error sending transaction: {:?}", e),
        }
    }

    Ok(())
}