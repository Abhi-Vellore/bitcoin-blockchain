use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};
use crate::blockchain::Blockchain;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use crate::types::block::{Block};
use std::collections::HashMap;
use log::{debug, warn, error};


#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}



impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        let mut orphan_buffer: HashMap<H256, Vec<Block>> = HashMap::new();
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                // PING
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }

                // PONG
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }

                // NEW BLOCK HASHES
                Message::NewBlockHashes(hashes) => {
                    // If not already hashed, then new block hashes
                    let blockchain = self.blockchain.lock().unwrap();
                    let mut unknown = Vec::new();
                    for hash in hashes.iter() {
                        // Get the hash and check
                        let block_hash = blockchain.get_block(hash);
                        match block_hash {
                            Ok(_) => {()}
                            Err(_) => {
                                // Add the hash to the unknown vector
                                unknown.push(hash.clone());
                            }
                        }
                    }

                    drop(blockchain);

                    // Asking for hashes for the unknown
                    if !unknown.is_empty() {
                        peer.write(Message::GetBlocks(unknown));
                    }
                }

                // GET BLOCKS
                Message::GetBlocks(hashes) => {
                    let blockchain = self.blockchain.lock().unwrap();
                    let mut known = Vec::new();
                    
                    for hash in hashes.iter() {
                        let result = blockchain.get_block(hash);
                        match result {
                            Ok(block) => {
                                known.push(block.clone());
                            }
                            Err(_) => {()}
                        }
                    }

                    drop(blockchain);

                    if !known.is_empty() {
                        peer.write(Message::Blocks(known));
                    }
                }

                // BLOCKS
                Message::Blocks(blocks) => {
                    let mut blockchain = self.blockchain.lock().unwrap();
                    
                    let mut new_block_hashes = Vec::new();
                    let mut blocks = blocks.clone();

                    let mut i = 0;
                    while i < blocks.len() {
                        let block = &blocks[i].clone();

                        if !(block.hash() <= block.get_difficulty()) {
                            continue;
                        }

                        let result = blockchain.get_block(&block.hash());
                        match result {
                            Ok(_) => {()}
                            Err(_) => {
                                // Add the new block
                                let result2 = blockchain.insert(block);
                                match result2 {
                                    Ok(_) => {
                                        // successfully inserted into blockchain
                                        // Need to add block hash
                                        new_block_hashes.push(block.hash());
                                        
                                        // claim orphans if possible
                                        if let Some(orphans) = orphan_buffer.get(&block.hash()) {
                                            blocks.extend_from_slice(&orphans);
                                            orphan_buffer.remove(&block.hash());
                                        }
                                    }

                                    // parent of the block is not in blockchain
                                    Err(_) => {
                                        // add block into a waiting queue (orphan buffer)
                                        if let Some(orphans) = orphan_buffer.get_mut(&block.get_parent()) {
                                            // buffer already contains an array of orphans corresponding 
                                            // to this block's parent
                                            orphans.push(block.clone());
                                        } 
                                        else {
                                            // buffer does not contain an array of orphans corresponding 
                                            // to this block's parent, so create a new array of orphans
                                            let orphans = vec![block.clone()];
                                            orphan_buffer.insert(block.get_parent(), orphans);
                                        }
                                        // Use the getblocks to get the necessary blocks
                                        peer.write(Message::GetBlocks(vec![block.hash()]));
                                    }      
                                }
                            }
                        }
                        i += 1;     // Next block
                    }

                    if new_block_hashes.len() != 0 {
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
                    }
                }  
                _ => {
                    // Logic for all other unhandled variants or...
                    unimplemented!() // Or `todo!()` if you want to add logic later.
                }

            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}

#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
    let worker = Worker::new(1, msg_chan, &server, &blockchain);
    worker.start(); 

    let current_chain = blockchain.lock().unwrap();
    let longest = current_chain.all_blocks_in_longest_chain();
    (test_msg_sender, server_receiver, longest)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST