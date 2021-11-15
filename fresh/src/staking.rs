use super::{Fresh, SimpleCoin};
use orga::coins::*;
use orga::prelude::*;

#[derive(State, Query, Call, Client)]
pub struct FreshNet {
    pub accounts: SimpleCoin,
    staking: Staking,
}

impl FreshNet {
    #[call]
    pub fn delegate(&mut self, validator_addr: Address, amount: Amount<Fresh>) -> Result<()> {
        let voting_power = {
            let signer = self
                .context::<Signer>()
                .ok_or_else(|| orga::Error::App("No signer context available".to_string()))?
                .signer
                .ok_or_else(|| orga::Error::App("Delegate calls must be signed".to_string()))?;
                
            let mut sender = self.accounts.balances_mut().entry(signer)?.or_default()?;
            let coins = sender.take(amount)?;

            let mut validator = self.staking.validators.get_mut(validator_addr)?;
            validator.get_mut(signer)?.give(coins)?;

            (validator.balance() * self.staking.vp_per_coin)?
        };

        self.context::<Validators>()
            .ok_or_else(|| orga::Error::App("No validator context available".to_string()))?
            .set_voting_power(validator_addr, voting_power.value);

        Ok(())
    }

    #[query]
    pub fn delegations(&self, delegator_address: Address) -> Result<Vec<(Address, Amount<Fresh>)>> {
        self.staking.validators
            .iter()?
            .filter_map(|entry| {
                let (k, v) = match entry {
                    Err(e) => return Some(Err(e)),
                    Ok((k, v)) => (*k, v),
                };
                match v.get(delegator_address) {
                    Err(e) => Some(Err(e)),
                    Ok(d) => {
                        if d.balance() == Amount::zero() {
                            None
                        } else {
                            Some(Ok((k, d.balance())))
                        }
                    },
                }
            })
            .collect()
    }
}

impl InitChain for FreshNet {
    fn init_chain(&mut self, ctx: &InitChainCtx) -> Result<()> {
        self.accounts.init_chain(ctx)?;
        self.staking.vp_per_coin = Amount::one();
        Ok(())
    }
}

impl BeginBlock for FreshNet {
    fn begin_block(&mut self, _ctx: &BeginBlockCtx) -> Result<()> {
        let balance = self.staking.validators.balance();
        if balance != 0 {
            let block_reward: Amount<Fresh> = 10.into();
            let increase = ((balance + block_reward) / balance)?;
            self.staking.vp_per_coin = (self.staking.vp_per_coin / increase)?;
            self.staking.validators.give(Fresh::mint(block_reward))?;
        }

        Ok(())
    }
}

type Delegators = Pool<Address, Coin<Fresh>, Fresh>;
#[derive(State)]
pub struct Staking {
    pub vp_per_coin: Amount<Fresh>,
    pub validators: Pool<Address, Delegators, Fresh>,
}
