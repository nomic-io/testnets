#![feature(trivial_bounds)]
#![feature(min_specialization)]

mod client;
mod coin;
mod staking;

use clap::Parser;

use coin::{Fresh, SimpleCoin};
use orga::prelude::*;
use staking::FreshNet;

type App = SignerPlugin<NoncePlugin<FreshNet>>;

fn rpc_client() -> TendermintClient<App> {
    TendermintClient::new("http://localhost:26657").unwrap()
}

#[derive(Parser, Debug)]
#[clap(version = "0.1", author = "The Nomic Developers <hello@nomic.io>")]
pub struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    Init(InitCmd),
    Start(StartCmd),
    Send(SendCmd),
    Balance(BalanceCmd),
    Delegations(DelegationsCmd),
    Delegate(DelegateCmd),
}

impl Command {
    async fn run(&self) -> Result<()> {
        use Command::*;
        match self {
            Init(cmd) => cmd.run().await,
            Start(cmd) => cmd.run().await,
            Send(cmd) => cmd.run().await,
            Balance(cmd) => cmd.run().await,
            Delegate(cmd) => cmd.run().await,
            Delegations(cmd) => cmd.run().await,
        }
    }
}


#[derive(Parser, Debug)]
pub struct InitCmd {}

impl InitCmd {
    async fn run(&self) -> Result<()> {
        tokio::task::spawn_blocking(|| {
            Node::<App>::new("freshnet");
        })
        .await
        .map_err(|err| orga::Error::App(err.to_string()))?;
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct StartCmd {}

impl StartCmd {
    async fn run(&self) -> Result<()> {
        tokio::task::spawn_blocking(|| {
            Node::<App>::new("freshnet")
                .with_genesis(include_bytes!("../genesis.json"))
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .run()
        })
        .await
        .map_err(|err| orga::Error::App(err.to_string()))?;
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct SendCmd {
    to_addr: Address,
    amount: u64,
}

impl SendCmd {
    async fn run(&self) -> Result<()> {
        rpc_client()
            .accounts
            .transfer(self.to_addr, self.amount.into())
            .await
    }
}

#[derive(Parser, Debug)]
pub struct DelegationsCmd;

impl DelegationsCmd {
    async fn run(&self) -> Result<()> {
        let my_address = load_keypair().unwrap().public.to_bytes().into();

        type FreshQuery = <FreshNet as Query>::Query;

        let delegations = rpc_client()
            .query(FreshQuery::MethodDelegations(my_address, vec![]), |state| {
                state.delegations(my_address)
            })
            .await?;

        println!(
            "delegated to {} validator{}",
            delegations.len(),
            if delegations.len() == 1 { "" } else { "s" }
        );
        for (validator, amount) in delegations {
            println!("- {}: {} FRESH", validator, amount);
        }

        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct BalanceCmd;

impl BalanceCmd {
    async fn run(&self) -> Result<()> {
        let my_address = load_keypair().unwrap().public.to_bytes().into();

        type FreshQuery = <FreshNet as Query>::Query;
        type CoinQuery = <SimpleCoin as Query>::Query;

        let balance = rpc_client()
            .query(
                FreshQuery::FieldAccounts(CoinQuery::MethodBalance(my_address, vec![])),
                |state| state.accounts.balance(my_address),
            )
            .await?;

        println!("address: {}", my_address);
        println!("balance: {} FRESH", balance);

        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct DelegateCmd {
    validator_addr: String,
    amount: u64,
}

impl DelegateCmd {
    async fn run(&self) -> Result<()> {
        use std::convert::TryInto;
        let validator_addr: [u8; 32] = base64::decode(&self.validator_addr)
            .map_err(|_| orga::Error::App("invalid validator address".to_string()))?
            .try_into()
            .map_err(|_| orga::Error::App("invalid validator address".to_string()))?;
        rpc_client()
            .delegate(validator_addr.into(), self.amount.into())
            .await
    }
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    opts.cmd.run().await.unwrap();
}
