use clap::{Args, Parser, Subcommand};
use reqwest::{header, Client, Response, Url};
use colored::*;
use mime::Mime;
use std::{collections::HashMap, str::FromStr};
use anyhow::{anyhow, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get(Get),
    Post(Post)
}

#[derive(Debug, Args)]
struct Get {
    #[arg(value_parser = parse_url)]
    url: String
}

#[derive(Debug, Args)]
struct Post {
    #[arg(value_parser = parse_url)]
    url: String,
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>
}

#[derive(Clone, Debug)]
#[warn(dead_code)]
struct KvPair {
    k: String,
    v: String
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

fn print_status(res: &Response) {
    let status = format!("{:?} {}", res.version(), res.status()).blue();
    println!("{}\n", status);
}

fn print_headers(res: &Response) {
    for (name, value) in res.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    println!("\n");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => println!("{}", body)
    }
} 

fn get_content_type(res: &Response) -> Option<Mime> {
    res.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

async fn print_response(res: Response) -> Result<()> {
    print_status(&res);
    print_headers(&res);
    let mime = get_content_type(&res);
    let body = res.text().await?;
    print_body(mime, &body);
    Ok(())
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let res = client.get(&args.url).send().await?;
    print_response(res).await?;
    Ok(())
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let res = client.post(&args.url).json(&body).send().await?;
    print_response(res).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let result = match cli.command {
        Commands::Get(ref args) => get(client, args).await?,
        Commands::Post(ref args) => post(client, args).await?
    };

    Ok(result)
}