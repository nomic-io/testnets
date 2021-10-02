// use orga::client::Client;
use orga::client::Client;
use orga::coins::*;
use orga::encoding::{Decode, Encode};
use orga::prelude::*;

#[derive(Encode, Decode)]
pub struct Eerie;
impl Symbol for Eerie {}

#[derive(State, Call, Client, Query)]
pub struct SimpleCoin {
    balances: Map<Address, Coin<Eerie>>,
}

impl InitChain for SimpleCoin {
    fn init_chain(&mut self, _ctx: &InitChainCtx) -> Result<()> {
        // TODO: initial balances
        self.balances.insert([0; 32].into(), Eerie::mint(100))?;
        Ok(())
    }
}

impl SimpleCoin {
    #[call]
    pub fn transfer(&mut self, to: Address, amount: Amount<Eerie>) -> Result<()> {
        let signer = self
            .context::<Signer>()
            .ok_or_else(|| failure::format_err!("No signer context available"))?
            .signer
            .ok_or_else(|| failure::format_err!("Transfer calls must be signed"))?;

        let mut sender = self.balances.entry(signer)?.or_default()?;
        let coins = sender.take(amount)?;

        let mut receiver = self.balances.entry(to)?.or_default()?;
        receiver.give(coins).unwrap();

        Ok(())
    }

    pub fn balances_mut(&mut self) -> &mut Map<Address, Coin<Eerie>> {
        &mut self.balances
    }
}
