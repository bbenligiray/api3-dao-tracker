use crate::components::footer;
use crate::components::header;
use crate::components::panel;
use crate::components::target::staking_note;
use crate::nice;
use crate::screens::meta::{MetaProvider, PageMetaInfo};
use crate::state::AppState;
use sauron::prelude::*;
use serde::{Deserialize, Serialize};
use web3::types::U256;

#[derive(Debug, Serialize, Deserialize)]
pub struct Screen {
    /// server side state
    pub state: AppState,
}

impl Screen {
    pub fn new(state: AppState) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

pub fn no_node<T>() -> Node<T> {
    div(vec![], vec![])
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {}

impl Screen {
    pub fn rewards_coeff(&self) -> f64 {
        match &self.state.pool_info {
            Some(x) => x.clone().rewards_coeff,
            None => 1f64,
        }
    }
    pub fn render_supply(&self) -> Node<Msg> {
        match &self.state.circulation {
            Some(c) => {
                let stake_target: U256 = match &self.state.pool_info {
                    Some(x) => c.total_supply * x.stake_target / U256::exp10(26),
                    None => U256::from(0),
                };
                let minted = self.state.get_minted_total();
                let total_shares = self.state.get_shares_total();
                node! {
                    <div>
                        {panel::render("API3 Circulating Supply", "", node! {
                            <div id="api3-circulating-supply">
                                <strong class="big-title" title={nice::amount(c.circulating_supply, 18)}>
                                    {text(nice::ceil(c.circulating_supply, 18))}
                                    <span class="desktop-only">" tokens"</span>
                                </strong>
                                <h3 class="cell-title"> "Total Locked" </h3>
                                <strong title={nice::amount(c.total_locked, 18)}>
                                    {text(nice::ceil(c.total_locked, 18))}
                                    <span class="desktop-only">" tokens"</span>
                                </strong>
                            </div>
                        })}

                        <div class="dash-row" id="staking">
                            <div class="dash-col dash-col-2 cell-t">
                                <h3 class="cell-title"> "Total Staked" </h3>
                                <strong title={nice::amount(total_shares + minted, 18)}>
                                    {text(nice::ceil(total_shares + minted, 18))}
                                    " tokens"
                                </strong>
                            </div>
                            <div class="dash-col dash-col-2 cell-t">
                                <h3 class="cell-title"> "Staking Target" </h3>
                                <strong title={nice::amount(stake_target, 10)}>
                                    {text(nice::ceil(stake_target, 10))}
                                    " tokens"
                                </strong>
                            </div>
                        </div>
                        <div class="dash-row">
                            {staking_note(self.state.apr, stake_target, total_shares + minted)}
                        </div>
                    </div>
                }
            }
            None => div(vec![class("error")], vec![text("no supply info")]),
        }
    }

    pub fn render_locked(&self) -> Node<Msg> {
        match &self.state.circulation {
            Some(c) => node! {
                <div class="dash-row" id="api3-locked-tokens">
                    <div class="dash-col dash-col-4 cell-t">
                        <h3 class="cell-title"> "Locked by governance" </h3>
                        <strong title={nice::amount(c.locked_by_governance, 18)}>
                            {text(nice::ceil(c.locked_by_governance, 18))}
                            " tokens"
                        </strong>
                    </div>
                    <div class="dash-col dash-col-4 cell-t">
                        <h3 class="cell-title"> "Locked vestings" </h3>
                        <strong title={nice::amount(c.locked_vestings, 18)}>
                            {text(nice::ceil(c.locked_vestings, 18))}
                            " tokens"
                        </strong>
                    </div>
                    <div class="dash-col dash-col-4 cell-t">
                        <h3 class="cell-title"> "Locked rewards" </h3>
                        <strong title={nice::amount(c.locked_rewards, 18)}>
                            {text(nice::ceil(c.locked_rewards, 18))}
                            " tokens"
                        </strong>
                    </div>
                    <div class="dash-col dash-col-4 cell-t">
                        <h3 class="cell-title"> "Time Locked" </h3>
                        <strong title={nice::amount(c.time_locked, 18)}>
                            {text(nice::ceil(c.time_locked, 18))}
                            " tokens"
                        </strong>
                    </div>
                </div>
            },
            None => div(vec![class("error")], vec![text("no supply info")]),
        }
    }

    pub fn render_contracts(&self) -> Node<Msg> {
        match &self.state.circulation {
            Some(c) => panel::render(
                "API3 Smart Contracts",
                "",
                node! {
                    <div>
                        <ul>
                            <li>
                                <label class="cell-title">"API3 pool contract address: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_pool))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"API3 token contract address: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_token))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"Time-lock manager contract: "</label>
                                <div class="eth-address">{text(format!("{:?} ", c.addr_time_lock))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"Primary voting contract: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_primary_contract))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"Primary treasury agent: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_primary_treasury))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"Secondary voting contract: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_secondary_contract))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"Secondary treasury agent: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_secondary_treasury))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"V1 Treasury address: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_v1_treasury))}</div>
                            </li>
                            <li>
                                <label class="cell-title">"Convenience contract: "</label>
                                <div class="eth-address">{text(format!("{:?}", c.addr_convenience))}</div>
                            </li>
                        </ul>
                    </div>
                },
            ),
            None => div(vec![class("error")], vec![text("no contracts info")]),
        }
    }

    pub fn current_epoch(&self, divclass: &'static str) -> Node<Msg> {
        let minted = self.state.get_minted_total();
        let staked256 = self.state.get_shares_total() + minted;
        let staked = nice::dec(staked256, 18);
        let to_be_minted = staked * self.state.apr * self.rewards_coeff() / 52.0;

        let prev_epoch = self.state.epoch_index - 1;
        let tm = if let Some(ep) = self.state.epochs.get(&prev_epoch) {
            if let Some(pool_info) = &self.state.pool_info {
                ep.tm + pool_info.epoch_length
            } else {
                0
            }
        } else {
            0
        };
        panel::render(
            "Current Epoch",
            divclass,
            node! {
                <div>
                    <div class="cell-title">
                        <span class="darken">"Epoch #"</span>
                        {text(nice::int(self.state.epoch_index))}
                    </div>
                    <h2 class="stats-row">
                        "APR: "
                        <strong class="big-title">
                            { text(format!("{:.2}%", 100.0*self.state.apr)) }
                        </strong>
                    </h2>
                    <div class="stats-row m20">
                        <span class="darken cell-title">"Epoch Rewards: "</span>
                        <strong class="accent">
                            { text(format!("{:.4}%", 100.0*self.state.apr*self.rewards_coeff() / 52.0)) }
                        </strong>
                    </div>
                    <div class="stats-row">
                        <span class="darken cell-title">"Staked now: "</span>
                        <strong title={nice::amount(staked256, 18)}>
                            { text(nice::ceil(staked256, 18)) }
                        </strong>
                    </div>
                    <div class="stats-row">
                        <span class="darken cell-title">"Including rewards: "</span>
                        <strong title={nice::amount(minted, 18)}>
                            { text(nice::ceil(minted, 18)) }
                        </strong>
                    </div>
                    <div class="padded">
                        <div class="stats-row cell-title">
                            <strong>
                                " ~"
                                { text(nice::int(to_be_minted as u64)) }
                            </strong>
                            <span class="darken">
                                " API3 tokens to be minted "
                            </span>
                        </div>
                        <div class="stats-row darken cell-title">
                            { text(nice::date(tm)) }
                        </div>
                    </div>
                </div>
            },
        )
    }

    pub fn render_epoch(&self, epoch: u64, divclass: &'static str) -> Node<Msg> {
        if self.state.epochs.len() == 0 {
            return div(vec![], vec![]);
        }
        let prev_epoch = self.state.epoch_index - epoch;
        if let Some(ep) = self.state.epochs.get(&prev_epoch) {
            panel::render(
                "Previous Epoch",
                divclass,
                node! {
                    <div>
                        <div class="cell-title">
                            <span class="darken">"Epoch #"</span>
                            {text(nice::int(ep.index))}
                        </div>
                        <h2 class="stats-row">
                            "APR: "
                            <strong class="big-title">
                                { text(format!("{:.2}%", 100.0*ep.apr)) }
                            </strong>
                        </h2>
                        <div class="stats-row m20">
                            <span class="darken cell-title">"Epoch Rewards: "</span>
                            <strong class="accent">
                                { text(format!("{:.4}%", 100.0*ep.apr*self.rewards_coeff() / 52.0)) }
                            </strong>
                        </div>
                        <div class="stats-row">
                            <div class="darken cell-title">"Staked at the end of epoch: "</div>
                            <strong title={nice::amount(ep.total, 18)}>
                                { text(nice::ceil(ep.total, 18)) }
                            </strong>
                        </div>
                        <div class="padded">
                            <div class="stats-row cell-title">
                                <strong>
                                    { text(nice::int(nice::dec(ep.minted, 18))) }
                                </strong>
                                <span class="darken">
                                    " API3 tokens were minted"
                                </span>
                            </div>
                            <div class="stats-row darken cell-title">
                                { text(nice::date(ep.tm)) }
                            </div>
                        </div>
                    </div>
                },
            )
        } else {
            text("")
        }
    }
}

impl Component<Msg> for Screen {
    fn view(&self) -> Node<Msg> {
        node! {
            <div class="screen-home">
                { header::render("", &self.state) }
                <div class="inner">
                    <div class="centered">
                        <h1>"API3 DAO Tracker"</h1>
                        <p class="m20">
                            "API3 DAO currently involves "
                            <a href="./wallets">
                                { text(nice::int(self.state.wallets.len())) }
                                " members"
                            </a>
                            " participated in "
                            <a href="./votings">
                                { text(nice::int(self.state.votings.len())) }
                                " votings"
                            </a>
                        </p>
                        <div style="height: 20px">" "</div>

                        <h2>"API3 Staking Rewards"</h2>
                        <div class="dash-row">
                            {self.current_epoch("dash-col dash-col-3")}
                            {self.render_epoch(1, "dash-col dash-col-3")}
                            {self.render_epoch(2, "dash-col dash-col-3")}
                        </div>

                        {match &self.state.circulation {
                            Some(_) => node!{
                                <div>
                                    <h2 class="m20">"API3 Token Supply"</h2>
                                    {self.render_locked()}
                                    <div class="dash-row">
                                        <div class="dash-col dash-col-2">
                                            {self.render_contracts()}
                                        </div>
                                        <div class="dash-col dash-col-2">
                                            {self.render_supply()}
                                        </div>
                                    </div>
                                </div>
                            },
                            None => text(""),
                        }}

                    </div>
                </div>
                { footer::render(&self.state) }
            </div>
        }
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        info!("MSG: {:?}", msg);
        Cmd::none()
    }
}

impl MetaProvider for Screen {
    fn meta(&self) -> PageMetaInfo {
        let title = format!("API3 DAO Tracker - on-chain analytics: members, staking rewards, API3 token circulating supply");
        let description = "API3 DAO tracker watches API3 on-chain DAO events, displays history of each participant and staking rewards. No wallet connection is needed";
        PageMetaInfo::new(&title, description)
    }
}
