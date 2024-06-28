use ethers::{
    types::U256,
    contract::abigen,
    core::types::{Address, Filter,TransactionRequest},
    providers::{Provider, Ws},
};
use ethers::prelude::*;
use eyre::Result;
use resolver::print;
use serde_json::Number;
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::time::{sleep, sleep_until, Duration, Instant};
use chrono::{Utc};
use hex_literal::hex;
use std::str;

type Client = SignerMiddleware<Provider<Http>, 
Wallet<k256::ecdsa::SigningKey>>;

abigen!(
    MyContract,
    r#"[
    event ActionExecutionAttempted(uint256 actionId, uint256 timeZero, address contractAddress)
    function executeAction(uint256 actionId) public returns (bool)
    ]"#,
);

abigen!(
    TransferAction,
    r#"[
        function getActionById(uint actionId) public view returns (address ownerAddress,bool initialized, uint duration, uint timeZero, bool isActive)
        function executeAction(uint actionId) public
    ]"#,
);



const WSS_URL: &str = "https://eth-sepolia.g.alchemy.com/v2/rOnL9TIb2mwbMDatQfBX-BJXUFH4Weml";
const TOKEN_DELEGATOR_CONTRACT_ADDRESS: &str="0x58816DfA47be3c6052c53605363395e74AF3a832";
const TRANSFER_ACTION_CONTRACT_ADDRESS: &str="0xDe8924e7B33c27e2b9Df2f54AFF11b5b0C6d7A16";
const PRIV_KEY:&str = "d9d238f2f5bd8e0e8f1436a747055c165ae04ffd1c00233f1134cbfa2c69ede3";

#[tokio::main]
async fn main() -> Result<()> {

    let provider = Provider::<Http>::try_from(WSS_URL)?;
    let wallet:LocalWallet = PRIV_KEY.parse::<LocalWallet>()?.with_chain_id(Chain::Sepolia);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet.clone()));
    let address: Address = TOKEN_DELEGATOR_CONTRACT_ADDRESS.parse()?;

    let latest_block = client.get_block_number().await?;
    println!("{} block", latest_block);

    listen_specific_events(&client, &address).await?;


    Ok(()) 
}


async fn listen_specific_events(client:&Client,contract_addr:&H160) -> Result<()> {

    let contract = MyContract::new(contract_addr.clone(), Arc::new(client.clone()));

    let transferAddress:Address = TRANSFER_ACTION_CONTRACT_ADDRESS.parse()?;
    let transferContract = TransferAction::new(transferAddress, Arc::new(client.clone()));
    let events = contract.event::<ActionExecutionAttemptedFilter>().from_block(6177567);
    let mut stream = events.stream().await?.take(1);
    
    while let Some(Ok(event)) = stream.next().await {
        println!("SomeEvent event: {event:?}");
        let mut time_zero = event.time_zero.as_u64();
        let action_id = event.action_id;
        
        loop{
            let now = SystemTime::now();
            let target_time = UNIX_EPOCH + Duration::from_secs(time_zero+5);

            if target_time <= now {
                println!("time_zero is in the past, immediately calling execute_action");
                let tx = contract.execute_action(U256::from(action_id).to_owned());
                let mut i = 0;
                loop{
                   match tx.send().await {
                       Ok(tx_result)=>{
                           println!("pending_tx {:?}", tx_result);
                           match tx_result.await {
                               Ok(tx_receipt) => {
                                   println!("mindex tx {:?}", tx_receipt);
                                   break Some(tx_receipt);
                               },
                               Err(e) => {
                                   println!("Failed to mine tx");
                                   println!("error {:?}",e);
                                       break None;
                               }
                           }
                       },
                       Err(e) => {
                           println!("Failed in pending tx");
                           println!("error {:?}",e);
                           if i==5{
                               break None;
                           }
                       }
                   }
                   i+=1;
               };
               let action = transferContract.get_action_by_id(action_id.clone()).call().await?;
            let (address, initialized, duration,time_zero_from_block, is_active) = action;
            println!("im after sleep");
            if !is_active{
                break;
            }
            println!("new timestamp {:?}",time_zero_from_block);
            time_zero = time_zero_from_block.as_u64(); 
            continue;
            } 

            let duration_until_target = target_time.duration_since(now)?;
            let target_instant = Instant::now() + duration_until_target;

            println!("im before sleep");
            sleep_until(target_instant).await;
            
            let tx = contract.execute_action(U256::from(action_id).to_owned());
            println!("tx {:?}",tx);

            let mut i = 0;
             loop{
                match tx.send().await {
                    Ok(tx_result)=>{
                        println!("pending_tx {:?}", tx_result);
                        match tx_result.await {
                            Ok(tx_receipt) => {
                                println!("mindex tx {:?}", tx_receipt);
                                break Some(tx_receipt);
                            },
                            Err(e) => {
                                println!("Failed to mine tx");
                                println!("error {:?}",e);
                                    break None;
                            }
                        }
                    },
                    Err(e) => {
                        println!("Failed in pending tx");
                        println!("error {:?}",e);
                        if i==5{
                            break None;
                        }
                    }
                }
                i+=1;
            };

        //     if pending_tx.is_none(){
        //         continue;
        //     }


        //         let mut j = 0;


        //  loop {
        //      match pending_tx.await {
        //         Ok(tx_receipt) => {
        //             println!("mindex tx {:?}", tx_receipt);
        //             break Some(tx_receipt);
        //         },
        //         Err(e) => {
        //             println!("Failed to send transaction");
        //             if j==5{
        //                 break None;
        //             }
        //         }
        //     }
        //     j+=1;
        // };

            

            let action = transferContract.get_action_by_id(action_id.clone()).call().await?;
            let (address, initialized, duration,time_zero_from_block, is_active) = action;
            println!("im after sleep");
            if !is_active{
                break;
            }
            println!("new timestamp {:?}",time_zero_from_block);
            time_zero = time_zero_from_block.as_u64();
         }
    }

    Ok(())
}
