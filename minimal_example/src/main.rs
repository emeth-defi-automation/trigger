use ethers::{
    types::U256,
    contract::abigen,
    core::types::{Address, Filter},
    providers::{Provider, Ws},
};
use ethers::prelude::*;
use eyre::Result;
use resolver::print;
use serde_json::Number;
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::time::{sleep_until, Instant, Duration};
use chrono::{Utc};
use hex_literal::hex;
use std::str;

type Client = SignerMiddleware<Provider<Http>, 
Wallet<k256::ecdsa::SigningKey>>;

abigen!(
    TokenManager,
    r#"[
    function setBalance(address user, uint amount) public
    function getBalance(address user) public view returns (uint)
    function transfer(address from, address to, uint amount) public returns (bool)
    ]"#,
);

abigen!(
    ActionExecutor,
    r#"[
    function executeTransfer(address from, address to, uint amount) public returns (bool)
    ]"#,
);


const WSS_URL: &str = "https://eth-sepolia.g.alchemy.com/v2/rOnL9TIb2mwbMDatQfBX-BJXUFH4Weml";
const TOKEN_MANAGER_CONTRACT_ADDRESS: &str="0x272447E0FD895158Ac2B1897e2EC9ccf1A6a8b5D";
const ACTION_EXECUTOR_CONTRACT_ADDRESS: &str="0x0C5c47ba5636f990E6c3b7e66CB52Dc611719EbB";
const PRIV_KEY:&str = "0x0b7239cc8c8b2626112cb5679db56cfcc73f6f2a0f8f24b81b84cc6eaf0bfb58";

#[tokio::main]
async fn main() -> Result<()> {

    let provider = Provider::<Http>::try_from(WSS_URL)?;
    let wallet:LocalWallet = PRIV_KEY.parse::<LocalWallet>()?.with_chain_id(Chain::Sepolia);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let token_manager_address: Address = TOKEN_MANAGER_CONTRACT_ADDRESS.parse()?;
    let executor_address: Address = ACTION_EXECUTOR_CONTRACT_ADDRESS.parse()?;

    // let contract = MyContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let token_manager = TokenManager::new(token_manager_address.clone(),Arc::new(client.clone()));
    let action_executor = ActionExecutor::new(executor_address.clone(),Arc::new(client.clone()));

    let tx1 = token_manager.set_balance("0x8C33f3Cd815e4C0624E53FadCf0fC21e19125bdD".parse()?,U256::from(1000)).send().await?.await?;
    println!("tx obj {:?}",tx1);
    let balance = token_manager.get_balance("0x8C33f3Cd815e4C0624E53FadCf0fC21e19125bdD".parse()?).call().await?;
    println!("balance of user1 {:?}",balance);
    let tx2 = token_manager.set_balance("0xF7829a3addC8BAD498fAa73370889c023D1295C5".parse()?,U256::from(6969)).send().await?.await?;
    println!("tx obj {:?}",tx2);
    let balance2 = token_manager.get_balance("0xF7829a3addC8BAD498fAa73370889c023D1295C5".parse()?).call().await?;
    println!("balance of user2 {:?}",balance2);

    let execute_transfer_tx = action_executor.execute_transfer("0x8C33f3Cd815e4C0624E53FadCf0fC21e19125bdD".parse()?, "0xF7829a3addC8BAD498fAa73370889c023D1295C5".parse()?, U256::from(20));
    println!("executied transfer {:?}",execute_transfer_tx);
    let sent_transfer = execute_transfer_tx.send().await?;
    println!("sent transfer {:?}",sent_transfer);
    let sent_transfer_tx = sent_transfer.await?;
    println!("sent transfer tx {:?}",sent_transfer_tx);


    let balance = token_manager.get_balance("0x8C33f3Cd815e4C0624E53FadCf0fC21e19125bdD".parse()?).call().await?;
    println!("balance of user1 {:?}",balance);
    let balance2 = token_manager.get_balance("0xF7829a3addC8BAD498fAa73370889c023D1295C5".parse()?).call().await?;
    println!("balance of user2 {:?}",balance2);

    Ok(()) 
}

