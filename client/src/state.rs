use crate::action::VotingAction;
use crate::events::{Api3, VotingAgent};
use crate::nice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web3::types::{H160, H256, U256};

// General API3 Pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api3PoolInfo {
    /// APR at genesis (min+max) / 2
    pub genesis_apr: f64,
    /// min APR
    pub min_apr: f64,
    /// max APR
    pub max_apr: f64,
    /// coefficient to apply to APR to generate rewards
    pub rewards_coeff: f64,
    /// length of epoch in seconds
    pub epoch_length: u64,
    /// number of epochs before rewards are unlocked
    pub reward_vesting_period: u64,
    /// staking target
    pub stake_target: U256,
    /// number of seconds before unstaking is allowed after claim
    pub unstake_wait_period: u64,
}

// General API3 Circulation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api3Circulation {
    /// tokens circulating supply
    pub circulating_supply: U256,
    /// total api3 token supply
    pub total_supply: U256,
    /// tokens, locked by governance
    pub locked_by_governance: U256,
    /// tokens, locked in rewards
    pub locked_rewards: U256,
    /// tokens, locked in vestings
    pub locked_vestings: U256,
    /// time locked tokens
    pub time_locked: U256,
    /// total locked tokens
    pub total_locked: U256,
    /// address of API3 pool contract
    pub addr_pool: H160,
    /// address of API3 token
    pub addr_token: H160,
    /// address of Time lock manager
    pub addr_time_lock: H160,
    /// address of API3 primary treasury
    pub addr_primary_treasury: H160,
    /// address of API3 secondary treasury
    pub addr_secondary_treasury: H160,
    /// address of V1 treasury
    pub addr_v1_treasury: H160,
    /// address of API3 primary voting contract
    pub addr_primary_contract: H160,
    /// address of API3 secondary voting contract
    pub addr_secondary_contract: H160,
    /// address of API3 convenience contract
    pub addr_convenience: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainEvent {
    pub entry: Api3,
    pub tm: u64,
    pub block_number: u64,
    pub tx: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingStaticData {
    pub start_date: u64,
    pub support_required: f64, // typically 0.5
    pub min_quorum: f64,       //typically 0.15 for secondary
    pub voting_power: U256,
    pub script: Vec<u8>,
    pub user_voting_power_at: U256,
    pub discussion_url: String,
}

impl VotingStaticData {
    pub fn into_details(&self) -> VotingDetails {
        VotingDetails {
            start_date: self.start_date,
            support_required: self.support_required,
            min_quorum: self.min_quorum,
            voting_power: self.voting_power,
            action: VotingAction::from_script(&self.script),
            user_voting_power_at: self.user_voting_power_at,
            discussion_url: self.discussion_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingDetails {
    pub start_date: u64,
    pub support_required: f64, // typically 0.5
    pub min_quorum: f64,       //typically 0.15 for secondary
    pub voting_power: U256,
    pub action: Option<VotingAction>,
    pub user_voting_power_at: U256,
    pub discussion_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Voting {
    pub primary: bool,
    pub vote_id: u64,
    pub tm: u64,
    pub block_number: u64,
    pub tx: H256,
    pub creator: H160,
    pub metadata: String,
    pub title: String,
    pub description: String,
    pub voted_yes: U256,
    pub voted_no: U256,
    pub yes: BTreeMap<H160, U256>,
    pub no: BTreeMap<H160, U256>,
    pub votes_total: U256,
    pub executed: bool,
    pub details: Option<VotingDetails>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vested_amount: Option<U256>,
    pub supporter: bool,
    pub deposited: U256,
    pub withdrawn: U256,
    pub staked: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_unstake: Option<ScheduledUnstake>,
    pub shares: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegates: Option<Delegation>,
    pub delegated: BTreeMap<H160, U256>,
    pub voting_power: U256,
    pub votes: u64,
    pub rewards: U256,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Wallet {
    pub fn get_name(&self) -> String {
        if let Some(ens) = &self.ens {
            return format!("{} ({:?})", ens.to_owned(), self.address);
        }
        format!("{:?}", self.address)
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    /// index of an epoch
    pub index: u64,
    /// APR during this epoch
    pub apr: f64,
    /// minted amount in the last MintedReward event
    pub minted: U256,
    /// Total stake during the last MintedReward event
    pub total: U256,
    /// Staking amount for each wallet (including locked rewards)
    pub stake: BTreeMap<H160, U256>,
    /// Timestamp of the epoch
    pub tm: u64,
    /// Block number of the epoch
    pub block_number: u64,
    /// Transaction of minting rewards
    pub tx: H256,
}

impl Epoch {
    pub fn new(
        index: u64,
        apr: f64,
        minted: U256,
        total: U256,
        stake: BTreeMap<H160, U256>,
        tm: u64,
        block_number: u64,
        tx: H256,
    ) -> Self {
        Self {
            index,
            apr,
            minted,
            total,
            stake,
            tm,
            block_number,
            tx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LabelBadge {
    pub class: String,
    pub text: String,
    pub title: String,
}

impl LabelBadge {
    pub fn new(class: &str, text: &str, title: &str) -> Self {
        Self {
            class: class.to_string(),
            text: text.to_string(),
            title: title.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    pub name: String,
    pub wallet: H160,
    pub balances: BTreeMap<String, U256>,
    pub updated_at: i64,
}

impl Treasury {
    pub fn new(name: String, wallet: H160) -> Self {
        Treasury {
            name: name.clone(),
            wallet: wallet.clone(),
            balances: BTreeMap::new(),
            updated_at: 0,
        }
    }

    pub fn update(&mut self, balances: BTreeMap<String, U256>) {
        self.balances = balances.clone();
        let dt = chrono::Utc::now().naive_utc();
        self.updated_at = dt.timestamp()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// version of the state
    pub version: String,
    /// chain ID
    pub chain_id: u64,
    /// current epoch index
    pub epoch_index: u64,
    /// current epoch APR
    pub apr: f64,
    /// the block of the last event
    pub last_block: u64,
    /// general API3 pool information
    pub pool_info: Option<Api3PoolInfo>,
    /// general API3 circulation information
    pub circulation: Option<Api3Circulation>,
    /// the map of epoch rewards
    //#[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub epochs: BTreeMap<u64, Epoch>,
    /// map of votings
    //#[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub votings: BTreeMap<u64, Voting>,
    /// log of events, grouped by votings
    //#[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub votings_events: BTreeMap<u64, Vec<OnChainEvent>>,
    /// map of wallets
    //#[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub wallets: BTreeMap<H160, Wallet>,
    /// log of events, groupped by wallets
    //#[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub wallets_events: BTreeMap<H160, Vec<OnChainEvent>>,
    /// list of wallets that are vesting and their balance is excluded from circulating supply
    pub vested: Vec<H160>,
    /// list of treasuries with their balances
    pub treasuries: BTreeMap<String, Treasury>,
    /// decimals for tokens
    pub decimals: BTreeMap<String, usize>,
    /// list of wallets that were in voting actions
    pub grants: BTreeMap<H160, u64>,
}

pub fn get_known_decimals() -> BTreeMap<String, usize> {
    let mut res = BTreeMap::new();
    res.insert("USDC".into(), 6);
    res.insert("API3".into(), 18);
    res
}

impl AppState {
    pub fn new(chain_id: u64) -> Self {
        let apr: f64 = 0.3875;
        Self {
            version: "20210820".to_owned(),
            chain_id,
            epoch_index: 1,
            apr,
            last_block: 0,
            epochs: BTreeMap::new(),
            votings: BTreeMap::new(),
            wallets: BTreeMap::new(),
            votings_events: BTreeMap::new(),
            wallets_events: BTreeMap::new(),
            vested: vec![],
            pool_info: None,
            circulation: None,
            treasuries: BTreeMap::new(),
            decimals: get_known_decimals(),
            grants: BTreeMap::new(),
        }
    }

    pub fn get_labels(&self, w: &Wallet) -> Vec<LabelBadge> {
        let mut labels: Vec<LabelBadge> = vec![];
        let vested = match &w.vested_amount {
            Some(amt) => *amt > U256::from(0),
            None => false,
        };

        if let Some(_) = self.grants.get(&w.address) {
            labels.push(LabelBadge::new(
                "badge-grant",
                "grant",
                "This address was a participant of voting as recipient",
            ));
        }

        if w.vested || self.is_vested_deposit(&w.address) {
            labels.push(LabelBadge::new(
                "badge-vested",
                "vested",
                "Some shares of this member are vested",
            ));
        }
        if !vested && w.supporter {
            labels.push(LabelBadge::new(
                "badge-supporter",
                "supporter",
                "API3 tokens are not vested, member can withdraw, but never did",
            ));
        }
        if w.withdrawn > U256::from(0) {
            labels.push(LabelBadge::new(
                "badge-withdrawn",
                "withdrawn",
                "Withdrew tokens in the past",
            ));
        } else if let Some(_) = w.scheduled_unstake {
            if w.withdrawn == U256::from(0) {
                labels.push(LabelBadge::new(
                    "badge-unstaking",
                    "unstaking",
                    "In the process of withdrawing",
                ));
            }
        } else if !w.supporter && w.deposited > U256::from(0) && w.voting_power == U256::from(0) {
            let delegates = match &w.delegates {
                Some(_) => true,
                None => false,
            };
            if !vested && !delegates {
                labels.push(LabelBadge::new(
                    "badge-not-staking",
                    "deposited, not staking",
                    "Deposited tokens but not staking them",
                ));
            }
        }
        if let Some(_) = &w.delegates {
            labels.push(LabelBadge::new(
                "badge-delegates",
                "delegates",
                "Delegates his stake to another member",
            ));
        }
        labels
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

    pub fn get_staked_for_epoch(&self, addr: &H160, epoch_index: u64) -> U256 {
        let ep = match self.epochs.get(&epoch_index) {
            Some(x) => x.clone(),
            None => return U256::from(0),
        };
        match ep.stake.get(&addr) {
            Some(x) => x.clone(),
            None => return U256::from(0),
        }
    }

    pub fn get_rewards_for_epoch(&self, addr: &H160, epoch_index: u64) -> U256 {
        if epoch_index > 1u64 {
            self.get_rewards(addr, epoch_index) - self.get_rewards(addr, epoch_index - 1)
        } else {
            U256::from(0)
        }
    }

    pub fn get_rewards(&self, addr: &H160, epoch_index: u64) -> U256 {
        self.epochs
            .iter()
            .map(|(_, epoch)| {
                let staked = match epoch.stake.get(addr) {
                    Some(val) => *val,
                    None => return U256::from(0),
                };
                if staked == U256::from(0) || epoch.index > epoch_index {
                    return U256::from(0);
                }

                // if *addr == hex_literal::hex!("6518c695cdcbefa272a4e5ef73bd46e801983e19").into() {
                //     println!("EPOCH {}", epoch.index);
                //     println!("epoch.minted {}", epoch.minted);
                //     println!("epoch.total {}", epoch.total);
                //     println!("staked {}", staked);
                // }

                (epoch.minted * staked) / epoch.total
            })
            .fold(U256::from(0), |a, b| a + b)
    }

    pub fn is_vested_deposit(&self, addr: &H160) -> bool {
        if let Some(w) = self.wallets.get(addr) {
            if let Some(vested) = w.vested_amount {
                return vested > U256::from(0);
            }
        }
        false
    }

    pub fn get_delegating_num(&self) -> u32 {
        self.wallets
            .values()
            .map(|w| match w.delegates {
                Some(_) => 1,
                None => 0,
            })
            .sum()
    }

    pub fn get_delegating_shares(&self) -> U256 {
        self.wallets
            .values()
            .map(|w| match w.delegates {
                Some(_) => w.shares,
                None => U256::from(0),
            })
            .fold(U256::from(0), |a, b| a + b)
    }

    // withdrawn more than 90% of their deposits
    pub fn get_withdrawn_num(&self) -> u32 {
        self.wallets
            .values()
            .map(
                |w| match w.withdrawn * U256::from(10) > w.deposited * U256::from(9) {
                    true => 1,
                    false => 0,
                },
            )
            .sum()
    }

    pub fn get_vested_num(&self) -> u32 {
        self.wallets
            .values()
            .map(|w| {
                if self.is_vested_deposit(&w.address) {
                    1
                } else {
                    0
                }
            })
            .sum()
    }

    pub fn get_vested_shares(&self) -> U256 {
        self.wallets
            .values()
            .map(|w| match w.vested_amount {
                Some(x) => x,
                None => U256::from(0),
            })
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

                if let None = w.vested_amount {
                    if w.withdrawn == U256::from(0) {
                        // if not vested and never withdrawn, mark as supporter
                        w.supporter = true
                    }
                }

                if let Some(d) = &mut w.delegates {
                    d.shares = w.shares;
                }
                w.update_voting_power();
                // returning shares are delegated
                (w.delegates.clone(), w.shares)
            }
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

    pub fn scheduled_unstake(
        &mut self,
        user: &H160,
        amount: &U256,
        shares: &U256,
        scheduled_for: u64,
    ) -> anyhow::Result<()> {
        // let total_stake = self.get_staked_total();
        // let total_shares = self.get_shares_total();
        let shares_to_unstake = *shares;
        // let mut amount_to_deduct = *shares * total_stake / total_shares;
        let mut amount_to_deduct = *amount;
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

                w.scheduled_unstake = Some(ScheduledUnstake {
                    amount: amount_to_deduct,
                    shares: shares_to_unstake,
                    tm: scheduled_for,
                });
                w.staked -= amount_to_deduct;
                w.shares -= shares_to_unstake;
                if let Some(d) = &mut w.delegates {
                    d.shares = w.shares;
                }
                w.supporter = false; // can't  be marked as supporter anymore
                w.update_voting_power();
                (w.delegates.clone(), w.shares)
            }
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
            let _ww = w.clone();
            let shares = *amount * total_shares / total_stake;

            match &mut w.scheduled_unstake {
                Some(scheduled) => {
                    if scheduled.shares < shares {
                        // warn!(
                        //     "unstaking shares {:?} amount {:?} was not scheduled, wallet {:?}",
                        //     shares, *amount, &ww
                        // )
                    }
                    w.scheduled_unstake = None;
                }
                None => {}
            };
        };
        Ok(())
    }

    pub fn distribute(
        &mut self,
        epoch_index: U256,
        amount: U256,
        new_apr: U256,
        total_stake: Option<U256>,
        tm: u64,
        block_number: u64,
        tx: H256,
    ) -> anyhow::Result<()> {
        let stake: BTreeMap<H160, U256> = self
            .wallets
            .iter()
            .map(|(addr, w)| (*addr, w.staked + w.rewards))
            .into_iter()
            .collect();
        let total = match total_stake {
            Some(x) => x - amount,
            None => stake.values().clone().fold(U256::from(0), |a, b| a + b),
        };
        let epoch: Epoch = Epoch::new(
            epoch_index.as_u64(),
            self.apr,
            amount,
            total,
            stake,
            tm,
            block_number,
            tx,
        );
        self.epochs.insert(epoch.index, epoch.clone());
        // distribute individual rewards
        self.wallets.iter_mut().for_each(|(_, w)| {
            let staked = w.staked + w.rewards;
            w.rewards += (epoch.minted * staked) / total;
        });

        // setting up new epoch
        self.epoch_index = epoch.index + 1;
        self.apr = nice::dec(new_apr, 14) * 0.0001;
        Ok(())
    }

    pub fn update(&mut self, e: OnChainEvent, log: web3::types::Log) -> () {
        log.block_number.map(|block_number| {
            self.last_block = block_number.as_u64();
        });

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
                if let Err(err) = self.distribute(
                    *epoch_index,
                    *amount,
                    *new_apr,
                    Some(*total_stake),
                    e.tm,
                    e.block_number,
                    e.tx,
                ) {
                    warn!("{:?} {:?}", err, e);
                }
            }
            Api3::MintedRewardV0 {
                epoch_index,
                amount,
                new_apr,
            } => {
                println!("{:?}", e.entry);
                if let Err(err) = self.distribute(
                    *epoch_index,
                    *amount,
                    *new_apr,
                    None,
                    e.tm,
                    e.block_number,
                    e.tx,
                ) {
                    warn!("{:?} {:?}", err, e);
                }
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
            Api3::DepositedVesting {
                user,
                amount,
                start: _,
                end: _,
                user_unstaked: _,
                user_vesting: _,
            } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.deposited += *amount;
                    w.vested_amount = Some(
                        *amount
                            + match w.vested_amount {
                                None => U256::from(0),
                                Some(v) => v,
                            },
                    );
                    w.supporter = false;
                }
            }
            Api3::DepositedByTimelockManager {
                user,
                amount,
                user_unstaked: _,
            } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.deposited += *amount;
                    w.supporter = false;
                }
            }
            Api3::Withdrawn {
                user,
                amount,
                user_unstaked: _,
            } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.withdrawn += *amount;
                    w.supporter = false; // can't  be marked as supporter anymore
                }
            }
            Api3::WithdrawnV0 { user, amount } => {
                if let Some(w) = self.wallets.get_mut(&user) {
                    w.withdrawn += *amount;
                    w.supporter = false; // can't  be marked as supporter anymore
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
                if let Err(err) =
                    self.scheduled_unstake(user, amount, shares, scheduled_for.as_u64())
                {
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
                if let Err(err) =
                    self.scheduled_unstake(user, amount, shares, scheduled_for.as_u64())
                {
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
            Api3::UnstakedV0 { user, amount } => {
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
                let parts: Vec<&str> = metadata.split("|").collect();
                let title = match parts.get(2) {
                    Some(x) => x.to_string(),
                    None => metadata.clone(),
                };
                let description = match parts.get(3) {
                    Some(x) => x.to_string(),
                    None => "".to_owned(),
                };
                let no: BTreeMap<H160, U256> = BTreeMap::new();
                let mut yes: BTreeMap<H160, U256> = BTreeMap::new();
                yes.insert(creator.clone(), self.get_voting_power_of(&creator));
                let v = Voting {
                    primary,
                    tm: e.tm,
                    block_number: e.block_number,
                    tx: e.tx,
                    vote_id: vote_id.as_u64(),
                    creator: creator.clone(),
                    metadata: metadata.clone(),
                    title,
                    description,
                    votes_total: self.get_votes_total(),
                    voted_yes: self.get_voting_power_of(&creator),
                    voted_no: U256::from(0),
                    yes,
                    no,
                    executed: false,
                    details: None,
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
                        v.yes.insert(voter.clone(), stake.clone());
                    } else {
                        v.no.insert(voter.clone(), stake.clone());
                    }
                    v.voted_yes = v
                        .yes
                        .iter()
                        .map(|(_, v)| v)
                        .fold(U256::from(0), |a, b| a + b);
                    v.voted_no =
                        v.no.iter()
                            .map(|(_, v)| v)
                            .fold(U256::from(0), |a, b| a + b);
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
                // println!("{:?}", e.entry);
                self.set_vesting_addresses(addresses);
            }
            _ => {}
        };
    }
}
