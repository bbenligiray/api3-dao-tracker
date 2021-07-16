pub mod args;
pub mod dumper;
pub mod endpoints;
pub mod ens;
pub mod inject;
pub mod reader;

use args::DumpMode;
use client::state::{AppState, OnChainEvent};
use futures::{FutureExt, StreamExt};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use web3::types::H160;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
type Subscribers = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

#[derive(Debug, Clone)]
pub struct State {
    /// whether to log incoming messages
    pub verbose: bool,
    /// subscribers of the chat
    pub subscribers: Subscribers,
    /// client application state
    pub app: AppState,
}

impl State {
    pub fn new(subscribers: Subscribers) -> Self {
        Self {
            subscribers,
            verbose: false,
            app: AppState::new(),
        }
    }
}

impl reader::EventHandler for State {
    fn on(&mut self, e: OnChainEvent, log: web3::types::Log) -> () {
        if self.verbose {
            // it becomes verbose in watching mode
            tracing::info!("{}", serde_json::to_string(&e).unwrap());
        }
        self.app.update(e.clone(), log);
        if self.verbose {
            futures::executor::block_on(async {
                let list = self.subscribers.read().await;
                // tracing::info!("sending to {:?} subscribers", list.len());
                // broadcasting event to all subscribers
                for (&subscriber_id, tx) in list.iter() {
                    let json_msg = serde_json::to_string(&e).unwrap();
                    tracing::debug!("<sent to #{}> {}", subscriber_id, json_msg);
                    if let Err(err) = tx.send(Ok(Message::text(json_msg))) {
                        tracing::warn!("<disconnected #{}> {}", subscriber_id, err);
                    }
                }
            });
        }
    }
}

async fn ws_connected(ws: WebSocket, subscribers: Subscribers) {
    // Use a counter to assign a new unique ID for this user.
    let subscriber_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    tracing::info!("connecting {}", subscriber_id);

    // Split the socket into a sender and receive of messages.
    let (ws_tx, mut ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            tracing::warn!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    subscribers.write().await.insert(subscriber_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the subscriber sends a message, broadcast it to
    // all other users...
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                tracing::warn!("websocket error(uid={}): {}", subscriber_id, e);
                break;
            }
        };
        tracing::debug!("message from user {:?}", msg);
    }

    // ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    ws_disconnected(subscriber_id, &subscribers).await;
}

async fn ws_disconnected(subscriber_id: usize, subscribers: &Subscribers) {
    // Stream closed up, so remove from the user list
    subscribers.write().await.remove(&subscriber_id);

    let s = subscribers.read().await;
    tracing::info!("disconnected {}, {} online", subscriber_id, s.len());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = match args::parse() {
        Ok(x) => x,
        Err(e) => return Err(anyhow::Error::msg(format!("Args parsing error {}", e))),
    };
    let addr_pool = H160::from_str(args.address_api3_pool.as_str()).expect("ADDR_API3_POOL");
    let addr_convenience =
        H160::from_str(args.address_convenience.as_str()).expect("ADDR_API3_CONVENIENCE");
    let addr_voting1 =
        H160::from_str(args.address_voting1.as_str()).expect("ADDR_API3_VOTING_PRIMARY");
    let addr_agent1 =
        H160::from_str(args.address_agent1.as_str()).expect("ADDR_API3_AGENT_PRIMARY");
    let addr_voting2 =
        H160::from_str(args.address_voting2.as_str()).expect("ADDR_API3_VOTING_SECONDARY");
    let addr_agent2 =
        H160::from_str(args.address_agent2.as_str()).expect("ADDR_API3_AGENT_SECONDARY");

    if let Some(_) = args.rpc_endpoint.find("http://") {
        return Err(anyhow::Error::msg(
            "only IPC endpoint is allowed. No real time events tracking with HTTP",
        ));
    } else if let Some(_) = args.rpc_endpoint.find("https://") {
        return Err(anyhow::Error::msg(
            "only IPC endpoint is allowed. No real time events tracking with HTTPS",
        ));
    }
    if !Path::new(args.rpc_endpoint.as_str()).exists() {
        return Err(anyhow::Error::msg("IPC file doesn't exists"));
    }
    let transport = web3::transports::Ipc::new(args.rpc_endpoint.as_str())
        .await
        .expect("Failed to connect to IPC");
    let web3 = web3::Web3::new(transport);

    let mut addresses = vec![addr_pool, addr_convenience];
    if let Some(addr_supply) = args
    .address_api3_supply
    .map(|x| H160::from_str(&x).expect("ADDR_API3_SUPPLY"))
    {
        addresses.push(addr_supply);
    }
    let scanner = reader::Scanner::new(
        args.cache_dir.as_str(),
        vec![addr_voting1, addr_agent1],
        vec![addr_voting2, addr_agent2],
        addresses,
        args.genesis_block,
        args.rpc_batch_size,
    );
    
    
    // starting a "loading" only server
    let socket_addr: std::net::SocketAddr = args.listen.parse().expect("invalid bind to listen");
    let loading_server = tokio::spawn(async move {
        let routes = endpoints::routes_loading();
        warp::serve(routes.with(warp::trace::request()))
            .run(socket_addr)
            .await;
    });

    if let Some(mode) = &args.dump {
        match mode {
            DumpMode::Unknown => {
                let mut dumper = dumper::Unknown::new();
                scanner.scan(&web3, &mut dumper).await?;
                dumper.done();
            }
            DumpMode::Events => {
                let mut dumper = dumper::Events::new();
                scanner.scan(&web3, &mut dumper).await?;
            }
        };
        std::process::exit(0);
    }

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let subscribers = Subscribers::default();
    let state = Arc::new(Mutex::new(State::new(subscribers.clone())));

    // Turn our "state" into a new Filter...
    let subscribers = warp::any().map(move || subscribers.clone());
    let last_block = {
        let rc = state.clone();
        let last_block = scanner.scan(&web3, &mut *rc.lock().unwrap()).await?;
        let s = rc.lock().unwrap();
        tracing::info!(
            "found: {} wallets, {} votings",
            s.app.wallets.len(),
            s.app.votings.len()
        );
        last_block
    };
    if !args.no_ens {
        let ens = crate::ens::ENS::new(web3.clone(), args.cache_dir.as_str());
        let rc = state.clone();
        let mut s = rc.lock().unwrap();
        for (addr, w) in &mut s.app.wallets {
            if let Some(name) = ens.name(addr.clone()).await {
                tracing::info!("ENS for {:?} is {:?}", addr, name);
                w.ens = Some(name);
            };
        }
    }

    loading_server.abort();
    std::thread::sleep(std::time::Duration::from_secs(1)); // wait for server to shutdown

    if args.watch {
        // This is unstable so far
        let rc = state.clone();
        rc.lock().unwrap().verbose = true;
        let rc = state.clone();
        tokio::spawn(async move {
            scanner.watch_ipc(&web3, last_block, rc).await.unwrap();
        });
        let chat = warp::path("ws").and(warp::ws()).and(subscribers).map(
            |ws: warp::ws::Ws, subscribers| {
                ws.on_upgrade(move |socket| ws_connected(socket, subscribers))
            },
        );
        let routes = endpoints::routes(args.static_dir.clone(), state).or(chat);
        warp::serve(routes.with(warp::trace::request()))
            .run(socket_addr)
            .await;
    } else {
        let routes = endpoints::routes(args.static_dir.clone(), state);
        warp::serve(routes.with(warp::trace::request()))
            .run(socket_addr)
            .await;
    }
    Ok(())
}
