use std::{sync::Arc, time::Duration};

use anyhow::Error;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
    cli_utils::Cli,
    conditional_engine::ConditionalEngine,
    http_client::HttpClient,
    pattern_utils::{Pattern, TemplateEngine},
    table_info::TableInfo,
};
mod banner;
mod cli_utils;
mod conditional_engine;
mod http_client;
mod pattern_utils;
mod table_info;

#[tokio::main]
async fn main() {
    let arg_matches = Cli::parse();
    banner::Banner::print_banner();
    // TODO: return iterator
    let users = match arg_matches.get_user_values() {
        Ok(users) => {
            if users.is_empty() {
                eprintln!("Error: users is empty");
                return;
            }
            users
        }
        Err(e) => {
            eprintln!("Error when getting users value with following error: {}", e);
            return;
        }
    };
    //TODO:: return iterator
    let passwords = match arg_matches.get_password_values() {
        Ok(passwords) => {
            if passwords.is_empty() {
                eprintln!("Error: passwords is empty");
                return;
            }
            passwords
        }
        Err(e) => {
            eprintln!(
                "Error when getting passwords value with following error: {}",
                e
            );
            return;
        }
    };
    let total_combination = (users.len() * passwords.len()) as u64;
    let progress_bar = Arc::new(ProgressBar::new(total_combination));
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let headers = match arg_matches.get_headers() {
        Ok(headers) => headers,
        Err(e) => {
            eprintln!(
                "Error when getting headers value with following error: {}",
                e
            );
            return;
        }
    };
    let url = arg_matches.url;
    let method = arg_matches.method;
    let template = arg_matches.template;
    let client = match HttpClient::default(&url) {
        Ok(client) => Arc::new(client),
        Err(e) => {
            eprintln!("Error when creating client with following error: {}", e);
            return;
        }
    };
    let concurrent = arg_matches.concurrent;

    let semaphore = Arc::new(Semaphore::new(concurrent));
    let rps = &arg_matches.rps;
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / rps));
    interval.tick().await;

    let mut set: JoinSet<Result<String, Error>> = JoinSet::new();
    let found = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let continue_on_success = arg_matches.continue_on_success;

    let conditional_string = arg_matches.conditional_string;
    let conditional_engine =
        match ConditionalEngine::new(&conditional_string).parse_original_string() {
            Ok(engine) => Arc::new(engine),
            Err(e) => {
                eprintln!(
                    "Error when parsing conditional string with following error: {}",
                    e
                );
                return;
            }
        };
    TableInfo::new(
        url.as_str(),
        method,
        template.as_str(),
        conditional_string.as_str(),
        rps.to_owned(),
        concurrent,
    )
    .print_info();

    //TODO: brute forse logic need to add as new module
    for user in users {
        for password in &passwords {
            if found.load(std::sync::atomic::Ordering::Relaxed) && !continue_on_success {
                break;
            }
            let template = template.clone();
            let semaphore = semaphore.clone();
            let user = user.clone();
            let password = password.clone();
            let headers = headers.clone();
            let client = client.clone();
            let mut engine = TemplateEngine::new();
            let found = found.clone();
            let progress_bar = progress_bar.clone();
            let conditional_engine = conditional_engine.clone();
            interval.tick().await;

            set.spawn(async move {
                engine.set_value(Pattern::User, user.as_str());
                engine.set_value(Pattern::Pass, password.as_str());
                engine.set_value(Pattern::RandIP, &engine.generate_random_ip());
                let headers = engine.render_headers(headers);

                let _permite = semaphore.acquire().await.unwrap();
                progress_bar.inc(1);
                match client
                    .send_request(method, engine.render(&template).as_str(), headers)
                    .await
                {
                    Ok(response) => {
                        let response = response;
                        let result = if let Ok(true) = conditional_engine.validate(response).await {
                            found.store(true, std::sync::atomic::Ordering::Relaxed);
                            format!("[SUCCESS] found {}:{}", user, password)
                        } else {
                            String::new()
                        };
                        Ok(result)
                    }
                    Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
                }
            });
        }
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok(success)) if !success.is_empty() => {
                println!("{}", success);
                if !continue_on_success {
                    set.abort_all();
                    break;
                }
            }
            Ok(Ok(_)) => {
                continue;
            }
            Ok(Err(e)) => {
                eprintln!("Task error: {}", e);
            }
            Err(e) if e.is_cancelled() => {
                continue;
            }
            Err(e) => {
                eprintln!("Join error: {}", e);
            }
        }
    }

    progress_bar.finish_with_message("Bruteforce completed!");
}
