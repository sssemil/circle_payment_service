use uuid::Uuid;

pub mod custody_type;
pub mod pagination;
pub mod public_key;
pub mod time_range;
pub mod transaction_transfer_create;
pub mod wallet_balance;
pub mod wallet_create;
pub mod wallet_detail;
pub mod wallet_get;
pub mod wallet_list;
pub mod wallet_nfts;
mod wallet_objects;
pub mod wallet_set;
pub mod wallet_update;
mod auth;

pub type RequestId = Uuid;
