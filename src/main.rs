// #![windows_subsystem = "windows"]
#![feature(generic_const_exprs)]

#[macro_use]
extern crate serde_derive;

use discord::{DiscordClientRequest, DiscordClientRequestType};
use single_instance::SingleInstance;
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;
use util::sleep;

mod config;
mod discord;
mod tray;
mod rank;
mod util;
mod melee;

#[tokio::main]
async fn main() {
    let instance = SingleInstance::new("SLIPPI_DISCORD_RICH_PRESENCE_MTX").unwrap();
    assert!(instance.is_single());
    let (tx, mut rx) = mpsc::channel::<DiscordClientRequest>(32);

    let cancel_token = CancellationToken::new();
    let melee_cancel_token = cancel_token.child_token();
    tokio::spawn(async move {
        loop {
            let melee_tx = tx.clone();
            let c_token = melee_cancel_token.clone();
            let res = tokio::task::spawn_blocking(move || {
                let mut client = melee::MeleeClient::new();
                client.run(c_token, melee_tx);
            }).await;
            match res {
                Ok(_) => { /* handle successfull exit */ },
                Err(err) if err.is_panic() => {
                    // panic
                    let _ = tx.send(DiscordClientRequest::clear()).await;
                    println!("[ERROR] Melee Client crashed. Restarting...");
                    sleep(500);
                },
                Err(_) => { return; }
            }
        }
    });

    let discord_cancel_token = cancel_token.clone();
    tokio::spawn(async move {
        let mut discord_client = discord::start_client().unwrap();

        loop {
            if discord_cancel_token.is_cancelled() {
                break
            }
            let poll_res = rx.try_recv();
            if poll_res.is_ok() {
                let msg = poll_res.unwrap();
                println!("{:?}", msg);
                match msg.req_type {
                    DiscordClientRequestType::Queue => discord_client.queue(msg.scene, msg.character).await,
                    DiscordClientRequestType::Game => discord_client.game(msg.stage, msg.character, msg.mode, msg.timestamp, msg.opp_name),
                    DiscordClientRequestType::Clear => discord_client.clear()
                }
            }
            
        }
        discord_client.close();
    });

    tray::run_tray(); // synchronous

    // cleanup
    cancel_token.cancel();
}