// use orga::client::Client;
use orga::client::Client;
use orga::coins::*;
use orga::encoding::{Decode, Encode};
use orga::prelude::*;

#[derive(Encode, Decode, Debug)]
pub struct Eerie;
impl Symbol for Eerie {}

#[derive(State, Call, Client, Query)]
pub struct SimpleCoin {
    balances: Map<Address, Coin<Eerie>>,
}

impl InitChain for SimpleCoin {
    fn init_chain(&mut self, _ctx: &InitChainCtx) -> Result<()> {
        // judd
        self.balances.insert(
            "nomic1zgu24fl96sf3epzvrwjr94nzw9yfsvququkxlk6v7wr3nazlzvnsudfdnq".parse()?,
            Eerie::mint(Amount::units(1000)),
        )?;
        
        // matt
        self.balances.insert(
            "nomic1cg4t0gpmgn944jpa0dlxa9ke7hz94vajk0qkkasdwhp7e074jx2qktweh2".parse()?,
            Eerie::mint(Amount::units(1000)),
        )?;

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

    #[query]
    pub fn balance(&self, address: Address) -> Result<u64> {
        // TODO: return an &Coin<Eerie>
        let maybe_account = self.balances.get(address)?;
        let balance = maybe_account.map(|acc| acc.amount).unwrap_or_default();
        Ok(balance.value)
    }

    pub fn balances_mut(&mut self) -> &mut Map<Address, Coin<Eerie>> {
        &mut self.balances
    }
}
