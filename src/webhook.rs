use crate::command;
use crate::command::WebhookCommands;
use helius::types::{
  AccountWebhookEncoding, Cluster, CreateWebhookRequest, EditWebhookRequest, TransactionStatus,
  TransactionType, Webhook, WebhookType,
};
use helius::Helius;

async fn list(helius: Helius) -> anyhow::Result<()> {
  let webhooks: Vec<Webhook> = helius.get_all_webhooks().await?;
  if webhooks.is_empty() {
    return Ok(println!("no webhooks found"));
  }
  Ok(println!("{webhooks:#?}"))
}

async fn create(
  helius: Helius,
  url: String,
  devnet: bool,
  addresses: Vec<String>,
) -> anyhow::Result<()> {
  let webhook_type = if devnet { WebhookType::EnhancedDevnet } else { WebhookType::Enhanced };
  let req = CreateWebhookRequest {
    webhook_url: url,
    transaction_types: vec![TransactionType::Transfer],
    account_addresses: addresses,
    webhook_type,
    auth_header: None,
    txn_status: TransactionStatus::All,
    encoding: AccountWebhookEncoding::JsonParsed,
  };
  let response = helius.create_webhook(req).await?;
  Ok(println!("id {}", response.webhook_id))
}

async fn delete(helius: Helius, id: String) -> anyhow::Result<()> {
  helius.delete_webhook(&id).await?;
  Ok(())
}

async fn add(helius: Helius, id: String, mut addresses: Vec<String>) -> anyhow::Result<()> {
  let mut hook = helius.get_webhook_by_id(&id).await?;
  hook.account_addresses.append(&mut addresses);
  helius
    .edit_webhook(EditWebhookRequest {
      webhook_id: hook.webhook_id.clone(),
      webhook_url: hook.webhook_url,
      transaction_types: hook.transaction_types,
      account_addresses: addresses,
      webhook_type: hook.webhook_type,
      auth_header: None,
      txn_status: hook.txn_status,
      encoding: hook.encoding,
    })
    .await?;
  Ok(())
}

pub(crate) async fn process_webhook(args: command::WebhookArgs) -> anyhow::Result<()> {
  let helius = Helius::new_with_async_solana(&args.helius_api_key, Cluster::MainnetBeta)?;
  match args.command {
    None => list(helius).await,
    Some(WebhookCommands::List) => list(helius).await,
    Some(WebhookCommands::Create(args)) => {
      create(helius, args.url, args.devnet, args.addresses).await
    },
    Some(WebhookCommands::Add(args)) => add(helius, args.id, args.addresses).await,
    Some(WebhookCommands::Delete(args)) => delete(helius, args.id).await,
  }
}
