// use orga::client::Client;
use orga::client::Client;
use orga::coins::*;
use orga::prelude::*;

#[derive(State, Encode, Decode, Debug)]
pub struct Fresh();
impl Symbol for Fresh {}

#[derive(State, Call, Client, Query)]
pub struct SimpleCoin {
    balances: Map<Address, Coin<Fresh>>,
}

impl InitChain for SimpleCoin {
    fn init_chain(&mut self, _ctx: &InitChainCtx) -> Result<()> {
        // matt
        self.balances.insert(
            "nomic1cg4t0gpmgn944jpa0dlxa9ke7hz94vajk0qkkasdwhp7e074jx2qktweh2".parse().unwrap(),
            ((100_000, std::marker::PhantomData),)
        )?;

        Ok(())
    }
}

impl SimpleCoin {
    #[call]
    pub fn transfer(&mut self, to: Address, amount: Amount<Fresh>) -> Result<()> {
        let signer = self
            .context::<Signer>()
            .ok_or_else(|| orga::Error::App("No signer context available".to_string()))?
            .signer
            .ok_or_else(|| orga::Error::App("Transfer calls must be signed".to_string()))?;

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

    pub fn balances_mut(&mut self) -> &mut Map<Address, Coin<Fresh>> {
        &mut self.balances
    }
}
