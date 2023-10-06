use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::{
    types::block::Block,
    blockchain::Blockchain,
    network::server::Handle as ServerHandle
};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan: finished_block_chan,
            blockchain: Arc::clone(blockchain)
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            // Receive from channel
            let block = self.finished_block_chan.recv().expect("Receive finished block error");
            
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            let mut blockchain = self.blockchain.lock().unwrap();
            let result = blockchain.insert(&block);
            drop(blockchain);
            
            match result {
                Ok(_) => println!("SUCCESS - inserted block into blockchain"),
                Err(e) => panic!("{}", e)
            }
        }
    }
}
