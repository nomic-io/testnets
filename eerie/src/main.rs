#![feature(trivial_bounds)]
#![feature(min_specialization)]

mod client;
mod coin;
mod staking;
use coin::{Eerie, SimpleCoin};

use orga::prelude::*;

type App = SignerProvider<NonceProvider<SimpleCoin>>;

#[tokio::main]
async fn main() {
    let mut client: TendermintClient<App> =
        TendermintClient::new("http://localhost:26657").unwrap();

    let args: Vec<String> = std::env::args().collect();
    match args
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [_, "node"] => {
            tokio::task::spawn_blocking(|| {
                Node::<App>::new("simple_coin")
                    .reset()
                    .run()
            })
            .await
            .unwrap();
        }
        [_, "balance"] => {
            let my_address = load_keypair().unwrap().public.to_bytes().into();
            let balance = client.query(
                <SimpleCoin as Query>::Query::MethodBalance(my_address, vec![]),
                |state| state.balance(my_address),
            ).await.unwrap();

            println!("address: {}", my_address);
            println!("balance: {} EERIE", balance);
        }
        [_, "send"] => {
            client.transfer([123; 32].into(), 5.into()).await.unwrap();
        }
        _ => {
            println!("hit catchall")
        }
    };
}
