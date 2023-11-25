use anyhow::Result;
use reqwest::Client;
use rsa::pkcs8::DecodePublicKey;
use rsa::sha2::Sha256;
use rsa::{Oaep, RsaPublicKey};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::CircleError;
use crate::models::public_key::PublicKeyResponse;
use crate::models::transaction::{TransactionRequest, TransactionResponse};
use crate::models::wallet_balance::{WalletBalanceQueryParams, WalletBalanceResponse};
use crate::models::wallet_create::{WalletCreateRequest, WalletCreateResponse};
use crate::models::wallet_set::{WalletSetRequest, WalletSetResponse};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiResponse<T> {
    data: T,
}

pub struct CircleClient {
    base_url: String,
    api_key: String,
    circle_entity_secret: String,
    client: Client,
    public_key: RsaPublicKey,
}

impl CircleClient {
    pub async fn new(api_key: String, circle_entity_secret: String) -> Result<Self> {
        let base_url = "https://api.circle.com/v1/".to_string();
        let client = Client::new();

        let url = format!("{}w3s/config/entity/publicKey", base_url);
        let res = client
            .get(&url)
            .header("Content-Type", "application/json")
            .bearer_auth(&api_key)
            .send()
            .await?;

        let public_key_response = if res.status().is_success() {
            res.json::<ApiResponse<PublicKeyResponse>>().await?.data
        } else {
            Err(CircleError::ResponseStatusCodeError(res.status()))?
        };

        let public_key_str = public_key_response.public_key.replace("RSA ", "");
        let public_key = RsaPublicKey::from_public_key_pem(&public_key_str).unwrap();

        Ok(CircleClient {
            base_url,
            api_key,
            circle_entity_secret,
            client: Client::new(),
            public_key,
        })
    }

    pub async fn create_wallet_set(
        &self,
        idempotency_key: Uuid,
        name: String,
    ) -> Result<WalletSetResponse> {
        let url = format!("{}w3s/developer/walletSets", self.base_url);
        let request = WalletSetRequest {
            idempotency_key,
            entity_secret_cipher_text: encrypt_entity_secret(
                &self.public_key,
                &self.circle_entity_secret,
            )?,
            name,
        };
        let res = self
            .client
            .post(&url)
            .json(&request)
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        if res.status().is_success() {
            let wallet_set_response = res.json::<ApiResponse<WalletSetResponse>>().await?;
            Ok(wallet_set_response.data)
        } else {
            Err(CircleError::ResponseStatusCodeError(res.status()))?
        }
    }

    pub async fn create_wallet(
        &self,
        idempotency_key: Uuid,
        wallet_set_id: Uuid,
        blockchains: Vec<String>,
        count: u32,
    ) -> Result<WalletCreateResponse> {
        let url = format!("{}w3s/developer/wallets", self.base_url);
        let request = WalletCreateRequest {
            idempotency_key,
            entity_secret_cipher_text: encrypt_entity_secret(
                &self.public_key,
                &self.circle_entity_secret,
            )?,
            wallet_set_id,
            blockchains,
            count,
        };
        let res = self
            .client
            .post(&url)
            .json(&request)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        if res.status().is_success() {
            let wallet_create_response = res.json::<ApiResponse<WalletCreateResponse>>().await?;
            Ok(wallet_create_response.data)
        } else {
            Err(CircleError::ResponseStatusCodeError(res.status()))?
        }
    }

    pub async fn get_wallet_balance(
        &self,
        wallet_id: Uuid,
        query_params: WalletBalanceQueryParams,
    ) -> Result<WalletBalanceResponse> {
        let url = format!("{}w3s/wallets/{}/balances", self.base_url, wallet_id);

        let res = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .query(&query_params)
            .send()
            .await?;

        if res.status().is_success() {
            let balance_response = res.json::<ApiResponse<WalletBalanceResponse>>().await?;
            Ok(balance_response.data)
        } else {
            Err(CircleError::ResponseStatusCodeError(res.status()))?
        }
    }

    pub async fn initiate_transaction(
        &self,
        request: TransactionRequest,
    ) -> Result<TransactionResponse> {
        let url = format!("{}w3s/developer/transactions/transfer", self.base_url);
        let res = self
            .client
            .post(&url)
            .json(&request)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        if res.status().is_success() {
            let transaction_response = res.json::<ApiResponse<TransactionResponse>>().await?;
            Ok(transaction_response.data)
        } else {
            Err(CircleError::ResponseStatusCodeError(res.status()))?
        }
    }
}

pub fn encrypt_entity_secret(public_key: &RsaPublicKey, entity_secret: &str) -> Result<String> {
    let entity_secret = hex::decode(entity_secret)?;
    let padding = Oaep::new::<Sha256>();
    let enc_data = public_key.encrypt(&mut rand::thread_rng(), padding, &entity_secret[..])?;
    Ok(base64::encode(enc_data))
}

#[cfg(test)]
mod test {
    use super::*;

    const PUBLIC_RSA_KEY_STR: &str = "-----BEGIN RSA PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAxDiWHMTzDfIMeLVw4BGT\nOnhVv/jjccrcHFMtm0ShbOb8bu0b/hvtN2oEdWx2RTdNT7AvntB9R9vCv60lZrk0\nZtfR8p2lew++NKAfyEeqTfL8dpfjhPtTZWLjdKG9SzkN66SRXBz5fNae4qaDHG3N\nI8PtYmwRnpfy6VzpcdwOGQxv2nGmgT4AKD/A1wl+7W2KruUPlWaGRNsSiFVceNTR\nYWll5OsRM0BB9YLkwDAFm27e+XmISJlapSmD8Gqx3i5ZvpwINboj1JiEaqMe/bAs\nASYHR73qz7G/B9p7nSc6tKr3SToXivZqDC47NLa81JZuyHyc7U5r+pdcTXOCsa+T\nTS0Y+fEZZ5rOQO3nI3voDULvf1yDvWsJTJW8qi3RjtGlR3P3M0JwONF0xZUwtSal\nMOLWwNjZrC33LIuGoD4M+43/y62xkdXIE4CHXTo3annRPnktkdYxTVfIYUXH8JDA\ng7++dIE4ZaN41Eg2mWCt3SSry9BqrMhEcY7YyuVyzJnv59cGCi5sDnQHGlXs1xJG\n/5QSyhID9+J2RRtu4sZ+5aLIvcIkMsNhul0mbfTRr34f9MGqYv9mkuzHUC/ppykG\nOv1ZJ0PWMIX4WCMXLKSi5Ii4Eayrev4BZk6WtXnvgX+EY9j+/85o+XgvyaX1Z7hE\nPBYZ9E8aCK/7kzIK4tgXviECAwEAAQ==\n-----END RSA PUBLIC KEY-----\n";

    #[tokio::test]
    async fn test_rsa_import() {
        use rsa::pkcs8::DecodePublicKey;
        use rsa::sha2::Sha256;
        use rsa::{Oaep, RsaPublicKey};

        let public_key_str = PUBLIC_RSA_KEY_STR.clone().replace("RSA ", "");
        let public_key = RsaPublicKey::from_public_key_pem(&public_key_str).unwrap();

        // Encrypt
        let data = b"hello world";
        let padding = Oaep::new::<Sha256>();
        let enc_data = public_key
            .encrypt(&mut rand::thread_rng(), padding, &data[..])
            .expect("failed to encrypt");
        assert_ne!(&data[..], &enc_data[..]);
    }

    #[tokio::test]
    async fn test_encrypt_hex_entity_secret() {
        let public_key_str = PUBLIC_RSA_KEY_STR.clone().replace("RSA ", "");
        let public_key = RsaPublicKey::from_public_key_pem(&public_key_str).unwrap();
        let dummy_entity_secret = hex::encode("test");
        encrypt_entity_secret(&public_key, &dummy_entity_secret).unwrap();
    }

    #[tokio::test]
    async fn test_parse_wallet_set_response() {
        let json = "{\"data\":{\"walletSet\":{\"id\":\"0068d5a4-eb64-4399-8441-a9af33af80a0\",\"custodyType\":\"DEVELOPER\",\"name\":\"test_wallet_set\",\"updateDate\":\"2023-11-25T14:26:38Z\",\"createDate\":\"2023-11-25T14:26:38Z\"}}}";
        let wallet_set_response =
            serde_json::from_str::<ApiResponse<WalletSetResponse>>(json).unwrap();
        assert_eq!(wallet_set_response.data.wallet_set.name, "test_wallet_set");
    }
}
