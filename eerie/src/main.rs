#![feature(trivial_bounds)]
#![feature(min_specialization)]

mod client;
mod coin;
mod staking;

use clap::{AppSettings, Clap};

use coin::{Eerie, SimpleCoin};
use orga::prelude::*;
use staking::EerieNet;

type App = SignerProvider<NonceProvider<EerieNet>>;

fn rpc_client() -> TendermintClient<App> {
    TendermintClient::new("http://localhost:26657").unwrap()
}

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "The Nomic Developers <hello@nomic.io>")]
pub struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum Command {
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
            Start(cmd) => cmd.run().await,
            Send(cmd) => cmd.run().await,
            Balance(cmd) => cmd.run().await,
            Delegate(cmd) => cmd.run().await,
            Delegations(cmd) => cmd.run().await,
        }
    }
}

#[derive(Clap, Debug)]
pub struct StartCmd {}

impl StartCmd {
    async fn run(&self) -> Result<()> {
        Ok(tokio::task::spawn_blocking(|| {
            Node::<App>::new("eerienet_data")
                .with_genesis("genesis.json")
                .run()
        })
        .await?)
    }
}

#[derive(Clap, Debug)]
pub struct SendCmd {
    to_addr: Address,
    amount: u64,
}

impl SendCmd {
    async fn run(&self) -> Result<()> {
        rpc_client().accounts.transfer(self.to_addr, self.amount.into()).await
    }
}

#[derive(Clap, Debug)]
pub struct DelegationsCmd;

impl DelegationsCmd {
    async fn run(&self) -> Result<()> {
        let my_address = load_keypair().unwrap().public.to_bytes().into();

        type EerieQuery = <EerieNet as Query>::Query;

        let delegations = rpc_client().query(
            EerieQuery::MethodDelegations(my_address, vec![]),
            |state| state.delegations(my_address),
        ).await?;

        println!("delegations: {}", delegations.len());
        for (validator, amount) in delegations {
            println!("{}: {} EERIE", validator, amount);
        }

        Ok(())
    }
}

#[derive(Clap, Debug)]
pub struct BalanceCmd;

impl BalanceCmd {
    async fn run(&self) -> Result<()> {
        let my_address = load_keypair().unwrap().public.to_bytes().into();

        type EerieQuery = <EerieNet as Query>::Query;
        type CoinQuery = <SimpleCoin as Query>::Query;

        let balance = rpc_client().query(
            EerieQuery::FieldAccounts(CoinQuery::MethodBalance(my_address, vec![])),
            |state| state.accounts.balance(my_address),
        ).await?;

        println!("address: {}", my_address);
        println!("balance: {} EERIE", balance);

        Ok(())
    }
}

#[derive(Clap, Debug)]
pub struct DelegateCmd {
    validator_addr: String,
    amount: u64,
}

impl DelegateCmd {
    async fn run(&self) -> Result<()> {
        use std::convert::TryInto;
        let validator_addr: [u8; 32] = base64::decode(&self.validator_addr)?
            .try_into()
            .map_err(|_| failure::format_err!("invalid validator address"))?;
        rpc_client().delegate(validator_addr.into(), self.amount.into()).await
    }
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    opts.cmd.run().await.unwrap();
}
