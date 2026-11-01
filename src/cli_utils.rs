use anyhow::{Error, Ok, anyhow};
use clap::{Args, Parser};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{path::PathBuf, str::FromStr};

use crate::http_client::HttpMethods;

#[derive(Args)]
#[command(next_help_heading = "Select login")]
#[group(id = "Users", required = true, multiple = false)]
pub struct UserArgs {
    #[arg(short, long, help = "Single username as LOGIN")]
    pub login: Option<String>,
    #[arg(short = 'L', long = "logins", help = "Path to wordlist with logins")]
    pub login_list: Option<PathBuf>,
}

#[derive(Args)]
#[command(next_help_heading = "Select Passwords")]
#[group(id = "Password", required = true, multiple = false)]
pub struct PasswordArgs {
    #[arg(short, long, help = "Single password as PASSWORD")]
    pub password: Option<String>,
    #[arg(
        short = 'P',
        long = "passwords",
        help = "Path to wordlist with passwords"
    )]
    pub password_list: Option<PathBuf>,
}

#[derive(Parser)]
#[command(name="hydr-oxide", next_line_help=true, override_usage="hydr-oxide [OPTIONS]",version="0.1.0",about,long_about=None,)]
pub struct Cli {
    #[arg(
        short,
        long,
        help = "Target url (ex. -u 'http(s)://exmaple.com/auth.php')"
    )]
    pub url: String,
    #[arg(short, long, help = "Request http method", default_value = "POST")]
    pub method: HttpMethods,

    #[arg(short = 'H', long, help = "Request header", num_args=1.., value_delimiter = ',')]
    pub headers: Option<Vec<String>>,
    #[arg(
        short,
        long,
        help = "Payload with template (ex. -t 'login=^USER^&pass=^PASS^)"
    )]
    pub template: String,

    #[arg(
        short = 'C',
        long,
        help = "Response validation rules in KEY::OP::VAL;... format",
        long_help = "A string of conditional rules to validate HTTP responses. Multiple rules can be chained using ';'.

Format:
  <KEY>::<OPERATOR>::<VALUE>;...

Available KEYS:
  C   - HTTP Response Status Code (CodeStatus)
  S   - Substring in the Response Body (ExpectedString)

Available OPERATORS:
  =   - Equal / Contains
  !=  - Not Equal / Does Not Contain
  >, >=, <, <=  - Numeric comparison (Only applicable for key 'C')

Examples:
  --conditional-string \"C::=::200\"
  --conditional-string \"C::>=::200;C::<::300\"
  --conditional-string \"C::!=::500;S::=::Welcome\""
    )]
    pub conditional_string: String,

    #[arg(short, long, help = "Request per second", default_value = "10")]
    pub rps: u64,

    #[arg(
        long,
        help = "Continue bruteforce even after finding valid credentials"
    )]
    pub continue_on_success: bool,

    #[arg(
        short = 'c',
        long,
        help = "Concurrent requests limit",
        default_value = "10"
    )]
    pub concurrent: usize,

    #[command(flatten)]
    pub user: UserArgs,
    #[command(flatten)]
    pub password: PasswordArgs,
}

impl Cli {
    pub fn get_user_values(&self) -> Result<Vec<String>, Error> {
        Self::get_values(self.user.login.clone(), self.user.login_list.clone())
    }

    pub fn get_password_values(&self) -> Result<Vec<String>, Error> {
        Self::get_values(
            self.password.password.clone(),
            self.password.password_list.clone(),
        )
    }

    pub fn get_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers_map = HeaderMap::new();
        if let Some(headers) = &self.headers {
            for header in headers {
                let (key, value) = header
                    .split_once(":")
                    .ok_or_else(|| anyhow!("Can't parse header: {}", header))?;
                let header_name = HeaderName::from_str(key.trim())?;
                let header_value = HeaderValue::from_str(value.trim())?;
                headers_map.insert(header_name, header_value);
            }
        }
        Ok(headers_map)
    }

    fn get_values(single: Option<String>, file: Option<PathBuf>) -> Result<Vec<String>, Error> {
        match (single, file) {
            (Some(value), None) => Ok(vec![value]),
            (None, Some(path)) => load_wordlist(&path),
            (Some(_), Some(_)) => Err(anyhow!("Cannot specify both single and file")),
            (None, None) => Err(anyhow!("Either value or file required")),
        }
    }
}

fn load_wordlist(path: &PathBuf) -> Result<Vec<String>, anyhow::Error> {
    let content = std::fs::read_to_string(path)?;
    Ok(content.lines().map(|s| s.trim().to_string()).collect())
}
