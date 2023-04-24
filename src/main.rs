// #![windows_subsystem = "windows"]

use discord::{DiscordClientRequest, DiscordClientRequestType};
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;
use util::sleep;

mod discord;
mod tray;
mod rank;
mod util;
mod melee;

#[tokio::main]
async fn main() {
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
                Ok(output) => { /* handle successfull exit */ },
                Err(err) if err.is_panic() => {
                    // panic
                    tx.send(DiscordClientRequest::clear()).await;
                    println!("[ERROR] Melee Client crashed. Restarting...");
                    sleep(500);
                },
                Err(err) => { return; }
            }
        }
    });

    let discord_cancel_token = cancel_token.clone();
    tokio::spawn(async move {
        let mut discord_client = discord::start_client().unwrap();
// discord_client.game(stage, character, mode);
        loop {
            if discord_cancel_token.is_cancelled() {
                break
            }
            let poll_res = rx.try_recv();
            if poll_res.is_ok() {
                let msg = poll_res.unwrap();
                println!("{:#?}", msg);
                match msg.req_type {
                    DiscordClientRequestType::Queue => discord_client.queue().await,
                    DiscordClientRequestType::Game => discord_client.game(msg.stage, msg.character, msg.mode, msg.timestamp),
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