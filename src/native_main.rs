#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    mcmc_demo::main(spawner).await;
}
