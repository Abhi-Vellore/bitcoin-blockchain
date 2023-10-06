pub mod worker;

use log::info;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
    time,
    thread,
};
use crate::blockchain::Blockchain;
use crate::types::{
    block::{Block, Content, Header},
    hash::Hashable,
    transaction::SignedTransaction,
    merkle::MerkleTree,
};

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain: &Arc<Mutex<Blockchain>>) -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: Arc::clone(blockchain),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    new(&blockchain)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual mining, create a block
            // TODO for student: if block mining finished, you can have something like: 
            // self.finished_block_chan.send(block.clone()).expect("Send finished block error");

            println!("Starting the Mining Process...");

            let blockchain = self.blockchain.lock().unwrap();

            let parent_hash = blockchain.tip();
            let parent_block = match blockchain.get_block(&parent_hash) {
                Ok(block) => block,    // parent exists in blockchain
                Err(_) => panic!("Parent node does not exist in blockchain."),   // parent not found
            };

            let difficulty = parent_block.get_difficulty();
            let mut rng = rand::thread_rng();

            drop(blockchain);

            let transactions: Vec<SignedTransaction> = Vec::new();  // empty transactions vector (for now)
            let merkle_tree = MerkleTree::new(&transactions.clone());
            let merkle_root = merkle_tree.root();
            
            // Loop to generate random nonces until desired hash is achieved
            while self.blockchain.lock().unwrap().tip() == parent_hash  {
                let content = Content{ 
                    transactions: transactions.clone() 
                };

                let nonce: u32 = rng.gen::<u32>();      // generate a random nonce
                
                let timestamp: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(time) => time.as_millis(),
                    Err(_) => panic!("SystemTime before UNIX EPOCH!"), 
                };
                
                let header = Header {
                    parent: parent_hash,
                    nonce: nonce,
                    difficulty: difficulty,
                    timestamp: timestamp,
                    merkle_root: merkle_root
                };

                let block = Block{ header, content };
                
                if block.hash() <= difficulty {
                    // Desired nonce found!
                    println!("Desired nonce found!");
                    println!("Parent Hash: {}", parent_hash);
                    println!("Block Hash : {}", block.hash());

                    // Insert block into blockchain (temporary)
                    match {self.blockchain.lock().unwrap().insert(&block)} {
                        Ok(_) => println!("SUCCESS - inserted block into blockchain"),
                        Err(e) => panic!("{}", e)
                    };

                    // Send to channel
                    self.finished_block_chan.send(block.clone()).expect("Sending to channel resulted in error.");
                    break;
                }
            }
            
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            
            // println!("PREV HASH: \t {}", block_prev.hash());
            // println!("PRNT HASH: \t {}", block_next.get_parent());

            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }

    #[test]
    #[timeout(60000)]
    fn miner_ten_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..9 {
            let block_next = finished_block_chan.recv().unwrap();
            
            // println!("PREV HASH: \t {}", block_prev.hash());
            // println!("PRNT HASH: \t {}", block_next.get_parent());

            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST