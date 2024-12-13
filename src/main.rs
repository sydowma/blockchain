// Cargo.toml 依赖:
// [dependencies]
// sha2 = "0.10.7"
// chrono = "0.4"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// hex = "0.4"

use sha2::{Sha256, Digest};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::fmt::Write;

// 交易结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: f64,
    timestamp: i64,
}

impl Transaction {
    fn new(sender: String, recipient: String, amount: f64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            timestamp: Utc::now().timestamp(),
        }
    }
}

// 区块结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: i64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap(),
            self.previous_hash,
            self.nonce
        );
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        let mut hash = String::new();
        for byte in result {
            write!(&mut hash, "{:02x}", byte).expect("Unable to write hash");
        }
        hash
    }

    fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while &self.hash[..difficulty] != target.as_str() {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }
}

// 区块链结构
#[derive(Debug, Serialize, Deserialize)]
struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: usize,
    mining_reward: f64,
}

impl Blockchain {
    fn new(difficulty: usize, mining_reward: f64) -> Self {
        let mut chain = vec![];
        // 创建创世区块
        let genesis_block = Block::new(
            0,
            vec![],
            "0".repeat(64),
        );
        chain.push(genesis_block);

        Blockchain {
            chain,
            pending_transactions: vec![],
            difficulty,
            mining_reward,
        }
    }

    fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    fn mine_pending_transactions(&mut self, miner_address: String) {
        // 创建挖矿奖励交易
        let reward_tx = Transaction::new(
            "System".to_string(),
            miner_address,
            self.mining_reward,
        );
        self.pending_transactions.push(reward_tx);

        // 创建新区块
        let mut block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            self.get_latest_block().hash.clone(),
        );

        // 挖矿
        block.mine_block(self.difficulty);

        // 将区块添加到链中
        println!("Block successfully mined!");
        self.chain.push(block);

        // 清空待处理交易池
        self.pending_transactions = vec![];
    }

    fn add_transaction(&mut self, sender: String, recipient: String, amount: f64) {
        let transaction = Transaction::new(sender, recipient, amount);
        self.pending_transactions.push(transaction);
    }

    fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.sender == address {
                    balance -= transaction.amount;
                }
                if transaction.recipient == address {
                    balance += transaction.amount;
                }
            }
        }

        balance
    }

    fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // 验证当前区块的哈希是否正确
            if current_block.hash != current_block.calculate_hash() {
                println!("Current hash is invalid");
                return false;
            }

            // 验证区块链接是否正确
            if current_block.previous_hash != previous_block.hash {
                println!("Chain link is broken");
                return false;
            }
        }
        true
    }
}

// 示例用法
fn main() {
    // 创建新的区块链，难度为4，挖矿奖励为100
    let mut blockchain = Blockchain::new(4, 100.0);

    println!("开始挖矿...");
    blockchain.mine_pending_transactions("miner1".to_string());

    // 添加一些交易
    blockchain.add_transaction("address1".to_string(), "address2".to_string(), 50.0);
    blockchain.add_transaction("address2".to_string(), "address3".to_string(), 30.0);

    println!("开始挖矿...");
    blockchain.mine_pending_transactions("miner1".to_string());

    // 查看余额
    println!("Miner1的余额是: {}", blockchain.get_balance("miner1"));
    println!("Address1的余额是: {}", blockchain.get_balance("address1"));
    println!("Address2的余额是: {}", blockchain.get_balance("address2"));
    println!("Address3的余额是: {}", blockchain.get_balance("address3"));

    // 验证区块链
    println!("区块链是否有效: {}", blockchain.is_chain_valid());

    // 将区块链序列化为JSON（用于持久化或网络传输）
    let blockchain_json = serde_json::to_string_pretty(&blockchain).unwrap();
    println!("区块链JSON:\n{}", blockchain_json);
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new(4, 100.0);
        assert_eq!(blockchain.chain.len(), 1); // 验证创世区块
        assert_eq!(blockchain.difficulty, 4);
        assert_eq!(blockchain.mining_reward, 100.0);
    }

    #[test]
    fn test_mining() {
        let mut blockchain = Blockchain::new(2, 100.0);
        blockchain.add_transaction("sender".to_string(), "recipient".to_string(), 50.0);
        blockchain.mine_pending_transactions("miner".to_string());
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_chain_validity() {
        let mut blockchain = Blockchain::new(2, 100.0);
        blockchain.add_transaction("sender".to_string(), "recipient".to_string(), 50.0);
        blockchain.mine_pending_transactions("miner".to_string());
        assert!(blockchain.is_chain_valid());
    }
}