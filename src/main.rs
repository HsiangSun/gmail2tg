use std::env;
use imap::Session;
use native_tls::TlsConnector;
use telegram_bot::*;
use std::fs::File;
use std::io::Write;
use chrono::{Duration, Utc};
use mailparse::MailHeaderMap;
use scraper::{Html, Selector};
use toml_env::Args;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod conf;

use crate::conf::myconf::ExternalConfig;
use crate::conf::singleton;

#[tokio::main]
async  fn main() -> Result<(),Box<dyn std::error::Error + Send + Sync> > {

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "gmail2tg=debug,reqwest=info,imap=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    //init config
    let conf :ExternalConfig  = toml_env::initialize(Args::default()).unwrap().unwrap();
    singleton::init_config(conf);

    let config = singleton::get_config();

    // 配置 IMAP 和 Telegram 参数
    let imap_server = &config.imap_server;
    let imap_port  =   &config.imap_port;
    let email_address = &config.listen_email_address;
    let email_password = &config.email_password;
    let telegram_token = &config.telegram_token;
    let telegram_chat_id = &config.telegram_chat_id;


    loop {
        // 创建 IMAP 连接
        let tls = TlsConnector::builder().build().unwrap();
        let client = imap::ClientBuilder::new(imap_server, 993).connect()?;
        // 检查新邮件
        let mut session = client
            .login(email_address, email_password)
            .map_err(|e| e.0)
            .unwrap();

        // 选择 INBOX 邮件箱
        session.select("INBOX").unwrap();

        // 获取过去 10 分钟的时间
        let ten_minutes_ago = Utc::now() - Duration::minutes(5);
        let since_date = ten_minutes_ago.format("%d-%b-%Y").to_string();

        // 搜索未读邮件和最近的邮件
        let query = format!("UNSEEN SINCE {}", since_date);
        // let query = "UNSEEN".to_string();

        let messages = session.search(query).unwrap();

        tracing::info!("UNREAD EMAIL SIZE {:?}", messages);

        for message_id in messages.iter() {
            if let Ok(message) = session.fetch(format!("{}", message_id), "RFC822") {
                if let Some(body) = message.iter().next() {
                    let email = body.body().expect("Failed to get email body");
                    let parsed_mail = mailparse::parse_mail(email).unwrap();

                    let from = parsed_mail.get_headers().get_first_value("From").unwrap();

                    tracing::debug!("From: {:?}", from);

                    if from.contains(&config.sender_email_address) {
                        let subject = parsed_mail
                            .get_headers()
                            .get_first_value("Subject")
                            .unwrap_or_default();

                        if subject.contains("[Sentry]") {
                            continue
                        }

                        // 提取正文和图片附件
                        let mut text_body = String::new();
                        let mut image_data = None;

                        for part in parsed_mail.subparts.iter() {
                            if let Some(content_type) =
                                part.get_headers().get_first_value("Content-Type")
                            {
                                if content_type.starts_with("text/html") {
                                    text_body = String::from_utf8(part.get_body_raw().unwrap())
                                        .unwrap();
                                } else if content_type.starts_with("image") {
                                    image_data = Some(part.get_body_raw().unwrap());
                                }
                            }
                        }

                        // println!("Subject: {}\n\n{}", subject, text_body);


                        // 解析 HTML
                        let document = Html::parse_document(&*text_body);

                        // 提取告警文本
                        let div_selector = Selector::parse("div").unwrap();
                        let alert_text = document
                            .select(&div_selector)
                            .next()
                            .map(|div| div.text().collect::<Vec<_>>().join(""))
                            .unwrap_or_default();

                        // 提取链接
                        let link_selector = Selector::parse("a").unwrap();
                        let link = document
                            .select(&link_selector)
                            .next()
                            .and_then(|a| a.value().attr("href"))
                            .unwrap_or("");


                        let public_link = link.replace("superset:8088", "superset.today");

                        if let Some(image) = image_data {
                            let mut file = File::create("temp_image.png").unwrap();
                            file.write_all(&image).unwrap();

                            let chat_id = ChatId::new(telegram_chat_id.to_owned());
                            let photo_file = InputFileUpload::with_path("temp_image.png");

                            // 创建按钮
                            let button = InlineKeyboardButton::url("在superset查看详情", public_link);
                            // let keyboard = InlineKeyboardMarkup::new();
                            let keyboard = InlineKeyboardMarkup::from(vec![vec![button]]);


                            // 发送图片并附加文字、按钮
                            let photo = InputFileUpload::with_path("temp_image.png");

                            let mut send_photo = SendPhoto::new(chat_id, photo_file);

                            let message = send_photo
                                .caption(alert_text)
                                .reply_markup(keyboard);





                            // 发送邮件正文到 Telegram
                            let api = Api::new(telegram_token);

                            match api.send(&message).await {
                                Ok(_) => {
                                    println!("Photo sent successfully with caption and button!");
                                    // 标记为已读
                                    session.store(format!("{}", message_id), "+FLAGS (\\Seen)").unwrap();
                                    tracing::info!("Make email:{} as read",message_id);
                                },
                                Err(e) => tracing::debug!("Failed to send photo: {:?}", e),
                            }

                        }
                    }
                }
            }
        }

        session.logout().unwrap();
        tracing::debug!("logout successful");

        // 等待一段时间再检查
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}
