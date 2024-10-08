use crate::metrics::Container;
use dashmap::DashMap;
use helius::types::{Cluster, EnhancedTransaction};
use helius::Helius;
use solana_sdk::pubkey::Pubkey;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use teloxide::prelude::*;
use teloxide::types::Recipient;
use tracing::{debug, error, info};
use warp::Filter;

pub struct SeleneBot {
  id: ChatId,
  bot: Bot,
  helius: Helius,
  name_cache: DashMap<String, AccountName>,
  metrics_container: Arc<Container>,
}

struct BotMessage(EnhancedTransaction);

struct BotMessages(Vec<BotMessage>);

#[derive(Clone)]
struct AccountName {
  pub address: String,
  pub name: String,
}

impl AccountName {
  pub fn new(address: String) -> Self {
    Self { address: address.clone(), name: address }
  }
}

impl Display for AccountName {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.address != self.name {
      write!(f, "{}={}", self.address, self.name)?;
    }
    Ok(())
  }
}

struct AccountNames(Vec<AccountName>);

impl Display for AccountNames {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let msg: Vec<String> = self.0.iter().map(|m| format!("{m}")).collect();
    writeln!(f, "{}", msg.join("\n"))
  }
}

impl Display for BotMessage {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\nhttps://xray.helius.xyz/tx/{}", self.0.description, self.0.signature)
  }
}

impl Display for BotMessages {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let msg: Vec<String> = self.0.iter().map(|m| format!("{m}")).collect();
    writeln!(f, "{}", msg.join("\n"))
  }
}

impl SeleneBot {
  pub fn new(
    id: i64,
    api_key: &str,
    token: String,
    metrics_container: Arc<Container>,
  ) -> anyhow::Result<Self> {
    let id = ChatId(id);
    let bot = Bot::new(token);
    let helius = Helius::new_with_async_solana(api_key, Cluster::MainnetBeta)?;

    let name_cache: DashMap<String, AccountName> = DashMap::new();
    Ok(Self { id, bot, helius, name_cache, metrics_container })
  }

  async fn get_name(&self, addr: &str) -> String {
    let start = Instant::now();
    info!("looking name for account {}", addr);
    self.metrics_container.measure(start, "get_name");
    String::new()
    /*
    let result = self.helius.get_names(addr).await;
    let name: String = match result {
      Ok(domain_names) => {
        if domain_names.domain_names.is_empty() {
          String::from(addr)
        } else {
          domain_names.domain_names.join(",")
        }
      },
      Err(err) => {
        error!("failed getting name for account {} {}", addr, err);
        String::from(addr)
      },
    };
    self.metrics_container.measure(start, "get_name");
    name
     */
  }

  async fn find_names(
    &self,
    transactions: &[EnhancedTransaction],
  ) -> anyhow::Result<Vec<AccountName>> {
    let mut names: Vec<AccountName> = transactions
      .iter()
      .flat_map(|t| t.account_data.iter())
      .map(|a| AccountName::new(String::from(&a.account)))
      .filter(|a| a.address != "11111111111111111111111111111111")
      .filter(|a| {
        let pk = Pubkey::from_str(&a.address);
        match pk {
          Ok(p) => p.is_on_curve(),
          Err(err) => {
            error!("{} is not a valid address {}", a.address, err);
            false
          },
        }
      })
      .collect();

    for account_name in names.iter_mut() {
      if self.name_cache.contains_key(&account_name.address) {
        account_name.name = self.name_cache.get(&account_name.address).unwrap().name.clone();
        continue;
      }
      let name = self.get_name(&account_name.address).await;
      self.metrics_container.cache_add();
      self.name_cache.insert(
        account_name.address.clone(),
        AccountName { address: account_name.address.clone(), name },
      );
    }
    self.metrics_container.cache(self.name_cache.len() as i64);
    Ok(names)
  }

  #[tracing::instrument(skip_all)]
  pub async fn handle_hook(&self, transactions: Vec<EnhancedTransaction>) {
    if transactions.is_empty() {
      return;
    }
    let start = Instant::now();
    let names: AccountNames =
      AccountNames(self.find_names(&transactions).await.unwrap_or_else(|_| Vec::new()));
    let messages: BotMessages = BotMessages(transactions.into_iter().map(BotMessage).collect());
    let to_send: String = format!("{}\n{}", messages, names);
    let r = Recipient::Id(self.id);
    let result = self.bot.send_message(r, &to_send).await;
    match result {
      Ok(message) => {
        info!("sent message");
        debug!("send_message result: {:#?}", message);
      },
      Err(err) => {
        error!("err:{:?}", err);
      },
    };
    self.metrics_container.measure(start, "handle_transactions");
  }

  #[tracing::instrument(skip(self))]
  pub async fn health(&self) -> anyhow::Result<u64> {
    let start = Instant::now();
    let height = self.helius.async_connection()?.get_block_height().await?;
    self.metrics_container.measure(start, "block_height");
    Ok(height)
  }
}

pub fn with_bot(
  bot: Arc<SeleneBot>,
) -> impl Filter<Extract = (Arc<SeleneBot>,), Error = Infallible> + Clone {
  warp::any().map(move || bot.clone())
}
