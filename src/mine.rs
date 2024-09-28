use base64::prelude::*;
use clap::{arg, Parser};
use drillx::equix;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    ops::{ControlFlow, Range},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        handshake::client::{generate_key, Request},
        Message,
    },
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub struct ServerMessagePoolSubmissionResult {
    difficulty: u32,
    total_balance: f64,
    total_rewards: f64,
    top_stake: f64,
    multiplier: f64,
    active_miners: u32,
    challenge: [u8; 32],
    best_nonce: u64,
    miner_supplied_difficulty: u32,
    miner_earned_rewards: f64,
    miner_percentage: f64,
}

impl ServerMessagePoolSubmissionResult {
    pub fn new_from_bytes(b: Vec<u8>) -> Self {
        let mut b_index = 1;

        let data_size = size_of::<u32>();
        let mut data_bytes = [0u8; size_of::<u32>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let difficulty = u32::from_le_bytes(data_bytes);

        let data_size = size_of::<f64>();
        let mut data_bytes = [0u8; size_of::<f64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let total_balance = f64::from_le_bytes(data_bytes);

        let data_size = size_of::<f64>();
        let mut data_bytes = [0u8; size_of::<f64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let total_rewards = f64::from_le_bytes(data_bytes);

        let data_size = size_of::<f64>();
        let mut data_bytes = [0u8; size_of::<f64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let top_stake = f64::from_le_bytes(data_bytes);

        let data_size = size_of::<f64>();
        let mut data_bytes = [0u8; size_of::<f64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let multiplier = f64::from_le_bytes(data_bytes);

        let data_size = size_of::<u32>();
        let mut data_bytes = [0u8; size_of::<u32>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let active_miners = u32::from_le_bytes(data_bytes);

        let data_size = 32;
        let mut data_bytes = [0u8; 32];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let challenge = data_bytes.clone();

        let data_size = size_of::<u64>();
        let mut data_bytes = [0u8; size_of::<u64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let best_nonce = u64::from_le_bytes(data_bytes);

        let data_size = size_of::<u32>();
        let mut data_bytes = [0u8; size_of::<u32>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let miner_supplied_difficulty = u32::from_le_bytes(data_bytes);

        let data_size = size_of::<f64>();
        let mut data_bytes = [0u8; size_of::<f64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        b_index += data_size;
        let miner_earned_rewards = f64::from_le_bytes(data_bytes);

        let data_size = size_of::<f64>();
        let mut data_bytes = [0u8; size_of::<f64>()];
        for i in 0..data_size {
            data_bytes[i] = b[i + b_index];
        }
        //b_index += data_size;
        let miner_percentage = f64::from_le_bytes(data_bytes);

        ServerMessagePoolSubmissionResult {
            difficulty,
            total_balance,
            total_rewards,
            top_stake,
            multiplier,
            active_miners,
            challenge,
            best_nonce,
            miner_supplied_difficulty,
            miner_earned_rewards,
            miner_percentage,
        }
    }

    pub fn _to_message_binary(&self) -> Vec<u8> {
        let mut bin_data = Vec::new();
        bin_data.push(1u8);
        bin_data.extend_from_slice(&self.difficulty.to_le_bytes());
        bin_data.extend_from_slice(&self.total_balance.to_le_bytes());
        bin_data.extend_from_slice(&self.total_rewards.to_le_bytes());
        bin_data.extend_from_slice(&self.top_stake.to_le_bytes());
        bin_data.extend_from_slice(&self.multiplier.to_le_bytes());
        bin_data.extend_from_slice(&self.active_miners.to_le_bytes());
        bin_data.extend_from_slice(&self.challenge);
        bin_data.extend_from_slice(&self.best_nonce.to_le_bytes());
        bin_data.extend_from_slice(&self.miner_supplied_difficulty.to_le_bytes());
        bin_data.extend_from_slice(&self.miner_earned_rewards.to_le_bytes());
        bin_data.extend_from_slice(&self.miner_percentage.to_le_bytes());

        bin_data
    }
}

#[derive(Debug)]
pub enum ServerMessage {
    StartMining([u8; 32], Range<u64>, u64),
    PoolSubmissionResult(ServerMessagePoolSubmissionResult),
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadSubmission {
    nonce: u64,
    difficulty: u32,
    pub d: [u8; 16], // digest
}

#[derive(Debug, Clone, Copy)]
pub enum MessageSubmissionProcessor {
    Submission(ThreadSubmission),
    Reset,
}

#[derive(Debug, Parser)]
pub struct MineArgs {
    #[arg(
        long,
        value_name = "CORES",
        default_value = "4",
        help = "Number of cores to use while mining"
    )]
    pub cores: u32,
}

const MIN_DIFF: u32 = 5; // MI, align with server

pub async fn mine(args: MineArgs, key: Keypair, url: String, unsecure: bool) {
    let running = Arc::new(AtomicBool::new(true));
    let key = Arc::new(key);

    loop {
        // MI: lan addr is allowed, example: 172.xx.xx.xxx:3000
        let mut ws_url_str = if unsecure {
            format!("ws://{}/v1/ws", url)
        } else {
            format!("wss://{}/v1/ws", url)
        };

        if ws_url_str.chars().last().unwrap() != '/' {
            ws_url_str.push('/');
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let ts_msg = timestamp.to_le_bytes();
        let sig = key.sign_message(&ts_msg);

        ws_url_str.push_str(&format!("?timestamp={}", timestamp));
        let url = url::Url::parse(&ws_url_str).expect("Failed to parse server url");
        let host = url.host_str().expect("Invalid host in server url");
        let mut threads = args.cores;

        let auth = BASE64_STANDARD.encode(format!("{}:{}", key.pubkey(), sig));

        println!("Connecting to server...");
        let request = Request::builder()
            .method("GET")
            .uri(url.to_string())
            .header("Sec-Websocket-Key", generate_key())
            .header("Host", host)
            .header("Upgrade", "websocket")
            .header("Connection", "upgrade")
            .header("Sec-Websocket-Version", "13")
            .header("Authorization", format!("Basic {}", auth))
            .body(())
            .unwrap();

        match connect_async(request).await {
            Ok((ws_stream, _)) => {
                println!("Connected to network!");

                let (sender, mut receiver) = ws_stream.split();
                let (message_sender, mut message_receiver) =
                    tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

                let receiver_thread = tokio::spawn(async move {
                    while let Some(Ok(message)) = receiver.next().await {
                        if process_message(message, message_sender.clone()).is_break() {
                            break;
                        }
                    }
                });

                let (solution_processor_message_sender, solution_processor_message_receiver) =
                    tokio::sync::mpsc::unbounded_channel::<MessageSubmissionProcessor>();

                let sender = Arc::new(Mutex::new(sender));
                let app_key = key.clone();
                let app_socket_sender = sender.clone();
                tokio::spawn(async move {
                    submission_processor(
                        app_key,
                        solution_processor_message_receiver,
                        app_socket_sender,
                    )
                    .await;
                });

                // send Ready message
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                let msg = now.to_le_bytes();
                let sig = key.sign_message(&msg).to_string().as_bytes().to_vec();
                let mut bin_data: Vec<u8> = Vec::new();
                bin_data.push(0u8);
                bin_data.extend_from_slice(&key.pubkey().to_bytes());
                bin_data.extend_from_slice(&msg);
                bin_data.extend(sig);

                let mut lock = sender.lock().await;
                let _ = lock.send(Message::Binary(bin_data)).await;
                drop(lock);

                // receive messages
                while let Some(msg) = message_receiver.recv().await {
                    tokio::spawn({
                        let message_sender = sender.clone();
                        let processor_submission_sender =
                            Arc::new(solution_processor_message_sender.clone());
                        let key = key.clone();
                        let running = running.clone();
                        async move {
                            if !running.load(Ordering::SeqCst) {
                                return;
                            }

                            match msg {
                                ServerMessage::StartMining(challenge, nonce_range, cutoff) => {
                                    println!(
                                        "\nMission received. New Challenge: {}",
                                        BASE64_STANDARD.encode(challenge)
                                    );
                                    println!(
                                        "Nonce range: {} - {}",
                                        nonce_range.start, nonce_range.end
                                    );
                                    println!("Start mining... will cutoff in: {}s", cutoff);
                                    let hash_timer = Instant::now();
                                    let core_ids = core_affinity::get_core_ids().unwrap();
                                    // let max_cores = std::thread::available_parallelism().unwrap().get();
                                    let max_cores = core_ids.len();
                                    if threads > max_cores as u32 {
                                        println!(
                                            "Arg --cores {} exceeds available cores({}), the exceeding part will be ignored.",
                                            threads, max_cores
                                        );

                                        threads = max_cores as u32
                                    }
                                    let nonces_per_thread = 10_000;
                                    let handles = core_ids
                                        .into_iter()
                                        .map(|i| {
                                            let running = running.clone(); // Capture running in thread
                                            let processor_submission_sender = processor_submission_sender.clone();
                                            std::thread::spawn({
                                                let mut memory = equix::SolverMemory::new();
                                                move || {
                                                    if (i.id as u32).ge(&threads) {
                                                        return None;
                                                    }

                                                    let _ = core_affinity::set_for_current(i);

                                                    let first_nonce = nonce_range.start
                                                        + (nonces_per_thread * (i.id as u64));
                                                    let mut nonce = first_nonce;
                                                    let mut best_nonce = nonce;
                                                    let mut best_difficulty = 0;
                                                    let mut best_hash = drillx::Hash::default();
                                                    let mut total_hashes: u64 = 0;

                                                    'challenge: loop {
                                                        // Check if Ctrl+C was pressed
                                                        if !running.load(Ordering::SeqCst) {
                                                            return None;
                                                        }

                                                        // Create hash
                                                        for hx in drillx::hashes_with_memory(
                                                            &mut memory,
                                                            &challenge,
                                                            &nonce.to_le_bytes(),
                                                        ) {
                                                            total_hashes += 1;
                                                            let difficulty = hx.difficulty();
                                                            if difficulty.gt(&best_difficulty) {
                                                                let thread_submission = ThreadSubmission {
                                                                    nonce,
                                                                    difficulty,
                                                                    d: hx.d,
                                                                };
                                                                let _ = processor_submission_sender.send(
                                                                    MessageSubmissionProcessor::Submission(
                                                                        thread_submission,
                                                                    ),
                                                                );
                                                                best_nonce = nonce;
                                                                best_difficulty = difficulty;
                                                                best_hash = hx;
                                                            }
                                                        }

                                                        // Exit if processed nonce range
                                                        if nonce >= nonce_range.end {
                                                            break;
                                                        }

                                                        if nonce % 100 == 0 {
                                                            if hash_timer.elapsed().as_secs().ge(&cutoff) {
                                                                if best_difficulty.ge(&MIN_DIFF) {
                                                                    break 'challenge;
                                                                }
                                                            }
                                                        }

                                                        // Increment nonce
                                                        nonce += 1;
                                                    }

                                                    // Return the best nonce
                                                    Some((
                                                        best_nonce,
                                                        best_difficulty,
                                                        best_hash,
                                                        total_hashes,
                                                    ))
                                                }
                                            })
                                        })
                                        .collect::<Vec<_>>();

                                    // Join handles and return best nonce
                                    let mut _best_nonce: u64 = 0;
                                    let mut best_difficulty = 0;
                                    let mut _best_hash = drillx::Hash::default();
                                    let mut total_nonces_checked = 0;
                                    for h in handles {
                                        if let Ok(Some((nonce, difficulty, hash, nonces_checked))) =
                                            h.join()
                                        {
                                            total_nonces_checked += nonces_checked;
                                            if difficulty > best_difficulty {
                                                best_difficulty = difficulty;
                                                _best_nonce = nonce;
                                                _best_hash = hash;
                                            }
                                        }
                                    }

                                    let hash_time = hash_timer.elapsed();

                                    println!(
                                        "✨ Mission completed! Best diff found: {}",
                                        best_difficulty
                                    );
                                    println!("Processed: {}", total_nonces_checked);
                                    println!("Hash time: {:?}", hash_time);
                                    if hash_time.as_secs().gt(&0) {
                                        println!(
                                            "Hashpower: {:?} H/s",
                                            total_nonces_checked
                                                .saturating_div(hash_time.as_secs())
                                        );
                                    } else {
                                        println!("Hashpower: {:?} H/s", 0);
                                    }

                                    let _ = processor_submission_sender
                                        .send(MessageSubmissionProcessor::Reset);

                                    // Ready up again
                                    let now = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards")
                                        .as_secs();

                                    let msg = now.to_le_bytes();
                                    let sig =
                                        key.sign_message(&msg).to_string().as_bytes().to_vec();
                                    let mut bin_data: Vec<u8> = Vec::new();
                                    bin_data.push(0u8);
                                    bin_data.extend_from_slice(&key.pubkey().to_bytes());
                                    bin_data.extend_from_slice(&msg);
                                    bin_data.extend(sig);
                                    {
                                        let mut message_sender = message_sender.lock().await;
                                        let _ =
                                            message_sender.send(Message::Binary(bin_data)).await;
                                    }
                                }
                                ServerMessage::PoolSubmissionResult(data) => {
                                    let message = format!(
                                        "\n\nPool Submitted Difficulty: {}\nPool Earned:  {:.11} ORE\nPool Balance: {:.11} ORE\nTop Stake:    {:.11} ORE\nPool Multiplier: {:.2}x\n----------------------\nActive Miners: {}\n----------------------\nMiner Submitted Difficulty: {}\nMiner Earned: {:.11} ORE\n{:.2}% of total pool reward\n",
                                        data.difficulty,
                                        data.total_rewards,
                                        data.total_balance,
                                        data.top_stake,
                                        data.multiplier,
                                        data.active_miners,
                                        data.miner_supplied_difficulty,
                                        data.miner_earned_rewards,
                                        data.miner_percentage
                                    );
                                    let _ = data.challenge;
                                    let _ = data.best_nonce;
                                    println!("{}", message);
                                }
                            }
                        }
                    });
                }

                let _ = receiver_thread.await;
            }
            Err(e) => {
                match e {
                    tokio_tungstenite::tungstenite::Error::Http(e) => {
                        if let Some(body) = e.body() {
                            println!("Error: {:?}", String::from_utf8(body.to_vec()));
                        } else {
                            println!("Http Error: {:?}", e);
                        }
                    }
                    _ => {
                        println!("Error: {:?}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

fn process_message(
    msg: Message,
    message_channel: UnboundedSender<ServerMessage>,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!("\n>>> Server Message: \n{}\n", t);
        }
        Message::Binary(b) => {
            let message_type = b[0];
            match message_type {
                0 => {
                    if b.len() < 49 {
                        println!("Invalid data for Message StartMining");
                    } else {
                        let mut hash_bytes = [0u8; 32];
                        // extract 256 bytes (32 u8's) from data for hash
                        let mut b_index = 1;
                        for i in 0..32 {
                            hash_bytes[i] = b[i + b_index];
                        }
                        b_index += 32;

                        // extract 64 bytes (8 u8's)
                        let mut cutoff_bytes = [0u8; 8];
                        for i in 0..8 {
                            cutoff_bytes[i] = b[i + b_index];
                        }
                        b_index += 8;
                        let cutoff = u64::from_le_bytes(cutoff_bytes);

                        let mut nonce_start_bytes = [0u8; 8];
                        for i in 0..8 {
                            nonce_start_bytes[i] = b[i + b_index];
                        }
                        b_index += 8;
                        let nonce_start = u64::from_le_bytes(nonce_start_bytes);

                        let mut nonce_end_bytes = [0u8; 8];
                        for i in 0..8 {
                            nonce_end_bytes[i] = b[i + b_index];
                        }
                        let nonce_end = u64::from_le_bytes(nonce_end_bytes);

                        let msg =
                            ServerMessage::StartMining(hash_bytes, nonce_start..nonce_end, cutoff);

                        let _ = message_channel.send(msg);
                    }
                }
                1 => {
                    let msg = ServerMessage::PoolSubmissionResult(
                        ServerMessagePoolSubmissionResult::new_from_bytes(b),
                    );
                    let _ = message_channel.send(msg);
                }
                _ => {
                    println!("Failed to parse server message type");
                }
            }
        }
        Message::Ping(v) => {
            println!("Got Ping: {:?}", v);
        }
        Message::Pong(v) => {
            println!("Got Pong: {:?}", v);
        }
        Message::Close(v) => {
            println!("Got Close: {:?}", v);
            return ControlFlow::Break(());
        }
        _ => {
            println!("Got invalid message data");
        }
    }

    ControlFlow::Continue(())
}

async fn submission_processor(
    key: Arc<Keypair>,
    mut processor_message_receiver: UnboundedReceiver<MessageSubmissionProcessor>,
    socket_sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
) {
    let mut best_diff = 0;
    while let Some(msg) = processor_message_receiver.recv().await {
        match msg {
            MessageSubmissionProcessor::Submission(thread_submission) => {
                if thread_submission.difficulty > best_diff {
                    best_diff = thread_submission.difficulty;

                    // Send results to the server
                    let message_type = 2u8; // 1 u8 - BestSolution Message
                    let best_hash_bin = thread_submission.d; // 16 u8
                    let best_nonce_bin = thread_submission.nonce.to_le_bytes(); // 8 u8

                    let mut hash_nonce_message = [0; 24];
                    hash_nonce_message[0..16].copy_from_slice(&best_hash_bin);
                    hash_nonce_message[16..24].copy_from_slice(&best_nonce_bin);
                    let signature = key
                        .sign_message(&hash_nonce_message)
                        .to_string()
                        .as_bytes()
                        .to_vec();

                    let mut bin_data = [0; 57];
                    bin_data[00..1].copy_from_slice(&message_type.to_le_bytes());
                    bin_data[01..17].copy_from_slice(&best_hash_bin);
                    bin_data[17..25].copy_from_slice(&best_nonce_bin);
                    bin_data[25..57].copy_from_slice(&key.pubkey().to_bytes());

                    let mut bin_vec = bin_data.to_vec();
                    bin_vec.extend(signature);

                    let mut message_sender = socket_sender.lock().await;
                    let _ = message_sender.send(Message::Binary(bin_vec)).await;
                    drop(message_sender);
                }
            }
            MessageSubmissionProcessor::Reset => {
                best_diff = 0;

                // Sleep for 2 seconds waiting for next mining mission.
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
