
pub mod myconf {
  #[derive(serde::Serialize, serde::Deserialize, Default,Debug)]
  pub struct Config {
    pub imap_server:            String,
    pub imap_port:              i32,
    pub listen_email_address:   String,
    pub sender_email_address:   String,
    pub email_password:         String,
    pub telegram_token:         String,
    pub telegram_chat_id:       i64,
  }

  pub use Config as ExternalConfig;

}

pub mod singleton;


