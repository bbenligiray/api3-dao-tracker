use crate::events::{Api3, VotingAgent};
use crate::nice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web3::types::{H160, H256, U256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainEvent {
    pub entry: Api3,
    pub tm: u64,
    pub block_number: u64,
    pub tx: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Voting {
    pub primary: bool,
    pub vote_id: u64,
    pub creator: H160,
    pub metadata: String,
    pub voted_yes: U256,
    pub voted_no: U256,
    pub list_yes: Vec<H160>,
    pub list_no: Vec<H160>,
    pub votes_total: U256,
    pub executed: bool,
}

impl Voting {
    pub fn as_u64(&self) -> u64 {
        let agent = if self.primary {
            VotingAgent::Primary
        } else {
            VotingAgent::Secondary
        };
        crate::events::voting_to_u64(&agent, self.vote_id)
    }
    pub fn key(&self) -> String {
        let agent = if self.primary {
            VotingAgent::Primary
        } else {
            VotingAgent::Secondary
        };
        crate::events::voting_to_string(&agent, self.vote_id)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Delegation {
    // adderss to which share are being delegated
    pub address: H160,
    // number of delegated shares
    pub shares: U256,
    // timestamp of the last delegation
    pub tm: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduledUnstake {
    // amount that is being unstaked
    pub amount: U256,
    // number of shares that are unstaking
    pub shares: U256,
    // timestamp of the last delegation
    pub tm: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wallet {
    pub address: H160,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ens: Option<String>,
    pub vested: bool,
    pub deposited: U256,
    #[serde(skip_serializing_if = "U256::is_zero")]
    pub withdrawn: U256,
    pub staked: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_unstake: Option<ScheduledUnstake>,
    pub shares: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegates: Option<Delegation>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub delegated: BTreeMap<H160, U256>,
    pub voting_power: U256,
    pub votes: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Wallet {
    pub fn update_voting_power(&mut self) {
        self.voting_power = {
            let mut sum = if let Some(_) = &self.delegates {
                // no voting power if there is a delegation
                U256::from(0)
            } else {
                self.shares
            };
            sum += self
                .delegated
                .values()
                .clone()
                .fold(U256::from(0), |a, b| a + b);
            sum
        };
    }
}

const APR_CORRECTION: f64 = 52.0 * 7.0 / 365.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    /// index of an epoch
    pub index: u64,
    /// APR during this epoch
    pub apr: f64,
    /// APY, calculated from APR
    pub apy: f64,
    /// minted amount in the last MintedReward event
    pub minted: U256,
    /// Total stake during the last MintedReward event
    pub total: U256,
    /// Staking amount for each wallet
    pub stake: BTreeMap<H160, U256>,
}

impl Epoch {
    pub fn new(
        index: u64,
        apr: f64,
        minted: U256,
        total_stake: Option<U256>,
        stake: BTreeMap<H160, U256>,
    ) -> Self {
        let total = match total_stake {
            Some(x) => x,
            None => stake.values().clone().fold(U256::from(0), |a, b| a + b),
        };
        Self {
            index,
            apr,
            apy: (1.0 + apr / 52.0).powf(52.0) - 1.0,
            minted,
            total,
            stake,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// current epoch index
    pub epoch_index: u64,
    /// current epoch APR
    pub apr: f64,
    /// current epoch APY
    pub apy: f64,
    /// the block of the last event
    pub last_block: u64,
    /// the map of epoch rewards
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub epochs: BTreeMap<u64, Epoch>,
    /// map of votings
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub votings: BTreeMap<u64, Voting>,
    /// log of events, grouped by votings
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub votings_events: BTreeMap<u64, Vec<OnChainEvent>>,
    /// map of wallets
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub wallets: BTreeMap<H160, Wallet>,
    /// log of events, groupped by wallets
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub wallets_events: BTreeMap<H160, Vec<OnChainEvent>>,
    /// list of wallets that are vesting and their balance is excluded from circulating supply
    pub vested: Vec<H160>,
}

impl AppState {
    pub fn new() -> Self {
        let apr: f64 = 0.3875 * APR_CORRECTION;
        Self {
            epoch_index: 1,
            apr,
            apy: (1.0 + apr / 52.0).powf(52.0) - 1.0,
            last_block: 0,
            epochs: BTreeMap::new(),
            votings: BTreeMap::new(),
            wallets: BTreeMap::new(),
            votings_events: BTreeMap::new(),
            wallets_events: BTreeMap::new(),
            vested: vec![],
        }
    }

    pub fn get_voting_power_of(&self, voter: &H160) -> U256 {
        match self.wallets.get(voter) {
            Some(wallet) => wallet.voting_power,
            None => U256::from(0),
        }
    }

    pub fn get_votes_total(&self) -> U256 {
        self.wallets
            .values()
            .map(|w| w.voting_power)
            .fold(U256::from(0), |a, b| a + b)
    }

    pub fn get_shares_total(&self) -> U256 {
        self.wallets
            .values()
            .map(|w| w.shares)
            .fold(U256::from(0), |a, b| a + b)
    }

    pub fn get_minted_total(&self) -> U256 {
        self.epochs
            .values()
            .map(|epoch| epoch.minted)
            .fold(U256::from(0), |a, b| a + b)
    }

    pub fn get_staked_total(&self) -> U256 {
        self.wallets
            .values()
            .map(|w| w.staked)
            .fold(U256::from(0), |a, b| a + b)
    }

    pub fn set_vesting_addresses(&mut self, addresses: &Vec<H160>) {
        self.wallets.iter_mut().for_each(|(addr, w)| {
            w.vested = addresses.contains(addr);
        });
        self.vested = addresses.clone();
    }

    pub fn delegate(&mut self, from: &H160, to: &H160, tm: u64) -> anyhow::Result<()> {
        let (address, delegates) = match self.wallets.get(from) {
            Some(x) => (x.clone().address, x.clone().delegates),
            None => return Err(anyhow::Error::msg("invalid from- wallet")),
        };
        // info!("delegated from={:?}, to: {:?}, shares: {:?}", from, to, shares);
        if let Some(existing) = &delegates {
            // remove existing delegation
            match self.wallets.get_mut(&existing.address) {
                Some(old) => {
                    let _ = old.delegated.remove(&address);
                    old.update_voting_power();
                }
                None => return Err(anyhow::Error::msg("no record of delegation wallet")),
            };
        }

        let w_from = {
            let w_from = match self.wallets.get_mut(from) {
                Some(x) => x,
                None => return Err(anyhow::Error::msg("invalid from- wallet")),
            };
            w_from.delegates = Some(Delegation {
                address: to.clone(),
                shares: w_from.shares,
                tm,
            });
            w_from.update_voting_power();
            w_from.clone() // releases self.wallets
        };

        // update record of "to"-wallet
        let w_to = match self.wallets.get_mut(to) {
            Some(x) => x,
            None => return Err(anyhow::Error::msg("invalid to- wallet")),
        };
        w_to.delegated.insert(address, w_from.shares);
        w_to.update_voting_power();
        Ok(())
    }

    pub fn undelegate(&mut self, from: &H160, to: &H160, shares: U256) -> anyhow::Result<()> {
        let delegates = match self.wallets.get(from) {
            Some(x) => x.clone().delegates,
            None => return Err(anyhow::Error::msg("invalid from- wallet")),
        };
        if let Some(existing) = &delegates {
            if existing.address != *to {
                return Err(anyhow::Error::msg("undelegate to doesn't match"));
            }
            // remove existing delegation
            match self.wallets.get_mut(&existing.address) {
                Some(old) => {
                    old.delegated.remove(&from);
                    old.update_voting_power();
                }
                None => return Err(anyhow::Error::msg("no record of delegation wallet")),
            };
        }

        let w_from = match self.wallets.get_mut(from) {
            Some(x) => x,
            None => return Err(anyhow::Error::msg("invalid from- wallet")),
        };
        if w_from.shares < shares {
            warn!("wallet {:?}", w_from);
            return Err(anyhow::Error::msg(format!(
                "shares amount {:?} is less than undelegated",
                w_from.shares
            )));
        }
        w_from.delegates = None;
        w_from.update_voting_power();
        Ok(())
    }

    pub fn staked(&mut self, user: &H160, amount: &U256, shares: &U256) -> anyhow::Result<()> {
        let (delegates, amt_delegated) = match self.wallets.get_mut(&user) {
            Some(w) => {
                w.staked += *amount;
                w.shares += *shares;
                if let Some(d) = &mut w.delegates {
                    d.shares = w.shares;
                }
                w.update_voting_power();
                // returning shares are delegated
                (w.delegates.clone(), w.shares)
            },
            None => (None, U256::from(0)),
        };

        if let Some(d) = delegates {
            if let Some(w) = self.wallets.get_mut(&d.address) {
                w.delegated.insert(*user, amt_delegated);
                w.update_voting_power();
            };
        }
        Ok(())
    }

    pub fn scheduled_unstake(&mut self, user: &H160, amount: &U256, shares: &U256, scheduled_for: u64) -> anyhow::Result<()> {
        let total_stake = self.get_staked_total();
        let total_shares = self.get_shares_total();
        let shares_to_unstake = *shares;
        let mut amount_to_deduct = *shares * total_stake / total_shares;
        let (delegates, amt_delegated) = match self.wallets.get_mut(user) {
            Some(w) => {
                let ww = w.clone();
                if w.shares < shares_to_unstake {
                    return Err(anyhow::Error::msg(format!(
                        "shares {:?} is less than trying to schedule for unstaking, amount {:?}, wallet {:?}",
                        shares_to_unstake, *amount, &ww,
                    )));
                }
                if w.staked < amount_to_deduct {
                    amount_to_deduct = w.staked; // what if we'll be fine...
                }
                if let Some(_) = w.scheduled_unstake {
                    // doesn't seems to happend
                    warn!("SHEDULED UNSTAKE TWICE {:?}", &ww);
                }

                w.scheduled_unstake = Some(ScheduledUnstake{
                    amount: amount_to_deduct,
                    shares: shares_to_unstake,
                    tm: scheduled_for,
                });
                w.staked -= amount_to_deduct;
                w.shares -= shares_to_unstake;
                if let Some(d) = &mut w.delegates {
                    d.shares = w.shares;
                }
                w.update_voting_power();
                (w.delegates.clone(), w.shares)
            },
            None => return Err(anyhow::Error::msg("invalid from- wallet")),
        };
        if let Some(d) = delegates {
            if let Some(w) = self.wallets.get_mut(&d.address) {
                w.delegated.insert(*user, amt_delegated);
                w.update_voting_power();
            };
        }
        Ok(())
    }

    pub fn unstaked(&mut self, user: &H160, amount: &U256) -> anyhow::Result<()> {
        let total_stake = self.get_staked_total();
        let total_shares = self.get_shares_total();
        if let Some(w) = self.wallets.get_mut(&user) {
            let ww = w.clone();
            let shares = *amount * total_shares / total_stake;

            match &mut w.scheduled_unstake {
                Some(scheduled) => {
                    if scheduled.shares != shares {
                        warn!("unstaking shares {:?} amount {:?} was not scheduled, wallet {:?}", shares, *amount, &ww)
                    }
                    w.scheduled_unstake = None;
                },
                None => {
                },
            };
        };
        Ok(())
    }

    pub fn update(&mut self, e: OnChainEvent, log: web3::types::Log) -> () {
        // println!("update {:?}", e);

        log.block_number.map(|block_number| {
            self.last_block = block_number.as_u64();
        });
        // self.events.push(e.clone());

        // if e.entry.is_broadcast() {
        //     self.wallets_events.iter_mut().for_each(|(_, w)| {
        //         w.push(e.clone());
        //     });
        // }

        e.entry.get_wallets().iter().for_each(|wallet| {
            if !self.wallets_events.contains_key(&wallet) {
                self.wallets_events.insert(wallet.clone(), vec![]);
                let mut w = Wallet::default();
                w.delegated = BTreeMap::new();
                w.address = wallet.clone();
                w.created_at = e.tm;
                self.wallets.insert(wallet.clone(), w);
            }
            if let Some(w) = self.wallets_events.get_mut(&wallet) {
                w.push(e.clone());
            }
            self.wallets.get_mut(&wallet).unwrap().updated_at = e.tm;
        });
        e.entry.get_voting().map(|id| {
            if !self.votings_events.contains_key(&id) {
                self.votings_events.insert(id, vec![]);
            }
            if let Some(v) = self.votings_events.get_mut(&id) {
                v.push(e.clone());
            }
        });
        match &e.entry {
            Api3::MintedReward {
                epoch_index,
                amount,
                new_apr,
                total_stake,
            } => {
                println!("{:?}", e.entry);
                let stake: BTreeMap<H160, U256> = self
                    .wallets
                    .iter()
                    .map(|(addr, w)| (*addr, w.staked))
                    .into_iter()
                    .collect();
                let epoch: Epoch = Epoch::new(
                    epoch_index.as_u64(),
                    self.apr,
                    *amount,
                    Some(*total_stake),
                    stake,
                );
                self.epochs.insert(epoch.index, epoch.clone());
                // setting up new epoch
                self.epoch_index = epoch.index + 1;
                self.apr = nice::dec(*new_apr, 14) * APR_CORRECTION * 0.0001;
                self.apy = (1.0 + self.apr / 52.0).powf(52.0) - 1.0;
            }
            Api3::MintedRewardV0 {
                epoch_index,
                amount,
                new_apr,
            } => {
                println!("{:?}", e.entry);
                let stake: BTreeMap<H160, U256> = self
                    .wallets
                    .iter()
                    .map(|(addr, w)| (*addr, w.staked))
                    .into_iter()
                    .collect();
                let epoch: Epoch = Epoch::new(epoch_index.as_u64(), self.apr, *amount, None, stake);
                self.epochs.insert(epoch.index.clone(), epoch.clone());
                // setting up new epoch
                self.epoch_index = epoch.index + 1;
                self.apr = nice::dec(*new_apr, 14) * APR_CORRECTION * 0.0001;
                self.apy = (1.0 + self.apr / 52.0).powf(52.0) - 1.0;
            }
            Api3::Deposited {
                user,
                amount,
                user_unstaked: _,
            } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.deposited += *amount;
                }
            }
            Api3::DepositedV0 { user, amount } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.deposited += *amount;
                }
            }
            Api3::Withdrawn {
                user,
                amount,
                user_unstaked: _,
            } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.withdrawn += *amount;
                }
            }
            Api3::WithdrawnV0 { user, amount } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.withdrawn += *amount;
                }
            }
            Api3::Staked {
                user,
                amount,
                minted_shares,
                user_unstaked: _,
                user_shares: _,
                total_shares: _,
                total_stake: _,
            } => {
                if let Err(err) = self.staked(user, amount, minted_shares) {
                    warn!("{:?} {:?}", err, e);  
                }
            }
            Api3::StakedV0 {
                user,
                amount,
                minted_shares,
            } => {
                if let Err(err) = self.staked(user, amount, minted_shares) {
                    warn!("{:?} {:?}", err, e);  
                }
            }
            Api3::ScheduledUnstake {
                user,
                amount,
                shares,
                scheduled_for,
                user_shares: _,
            } => {
                if let Err(err) = self.scheduled_unstake(user, amount, shares, scheduled_for.as_u64()) {
                    warn!("{:?} {:?}", err, e);
                    std::process::exit(0);
                }
            }
            Api3::ScheduledUnstakeV0 {
                user,
                amount,
                shares,
                scheduled_for,
            } => {
                if let Err(err) = self.scheduled_unstake(user, amount, shares, scheduled_for.as_u64()) {
                    warn!("{:?} {:?}", err, e);
                    std::process::exit(0);
                }
            }

            Api3::Unstaked {
                user,
                amount,
                user_unstaked: _,
                total_shares: _,
                total_stake: _,
            } => {
                if let Err(err) = self.unstaked(user, amount) {
                    warn!("{:?} {:?}", err, e);
                }
            }
            Api3::UnstakedV0 {
                user,
                amount,
            } => {
                if let Err(err) = self.unstaked(user, amount) {
                    warn!("{:?} {:?}", err, e);
                }
            }

            // You can't trust amount of shares from this event
            // as in the case of STAKE+DELEGATE, the order is broken,
            // and DELEGATE event comes first with amount that is not on stake yet.
            // https://rinkeby.etherscan.io/tx/0xb9eabaa1704a6a4b0c8d30e342d8fe11bb42c83452c00825ea3a011f9a823bf0#eventlog
            // Furthermore, when delegation happens, it applies to all amount,
            // including future shares changes
            Api3::Delegated {
                from,
                to,
                shares: _,
                total_delegated_to: _,
            } => {
                if let Err(err) = self.delegate(from, to, e.tm) {
                    warn!("{:?} {:?}", err, e);
                }
            }
            Api3::DelegatedV0 {
                from,
                to,
                shares: _,
            } => {
                if let Err(err) = self.delegate(from, to, e.tm) {
                    warn!("{:?} {:?}", err, e);
                }
            }
            Api3::Undelegated {
                from,
                to,
                shares,
                total_delegated_to: _,
            } => {
                if let Err(err) = self.undelegate(from, to, *shares) {
                    warn!("{:?} {:?}", err, e);
                }
            }
            Api3::UndelegatedV0 { from, to, shares } => {
                if let Err(err) = self.undelegate(from, to, *shares) {
                    warn!("{:?} {:?}", err, e);
                }
            }

            Api3::StartVote {
                agent,
                vote_id,
                creator,
                metadata,
            } => {
                let primary = match agent {
                    VotingAgent::Primary => true,
                    VotingAgent::Secondary => false,
                };
                let v = Voting {
                    primary,
                    vote_id: vote_id.as_u64(),
                    creator: creator.clone(),
                    metadata: metadata.clone(),
                    votes_total: self.get_votes_total(),
                    voted_yes: self.get_voting_power_of(&creator),
                    voted_no: U256::from(0),
                    list_yes: vec![creator.clone()],
                    list_no: vec![],
                    executed: false,
                };
                self.votings.insert(v.as_u64(), v);
                if let Some(w) = self.wallets.get_mut(&creator) {
                    w.votes = w.votes + 1;
                }
            }
            Api3::CastVote {
                agent,
                vote_id,
                voter,
                supports,
                stake,
            } => {
                let key = crate::events::voting_to_u64(agent, vote_id.as_u64());
                if let Some(v) = self.votings.get_mut(&key) {
                    if *supports {
                        v.voted_yes += *stake;
                        v.list_yes.push(voter.clone())
                    } else {
                        v.voted_no += *stake;
                        v.list_no.push(voter.clone())
                    }
                }
                if let Some(w) = self.wallets.get_mut(&voter) {
                    w.votes = w.votes + 1;
                }
            }
            Api3::ExecuteVote { agent, vote_id } => {
                let key = crate::events::voting_to_u64(agent, vote_id.as_u64());
                if let Some(v) = self.votings.get_mut(&key) {
                    v.executed = true;
                }
            }
            Api3::SetVestingAddresses { addresses } => {
                println!("{:?}", e.entry);
                self.set_vesting_addresses(addresses);
            }
            _ => {}
        };
    }
}
