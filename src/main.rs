use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    api_key: Option<String>,
    #[arg(long)]
    name: String,
    #[arg(long)]
    subdomain: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let ipv4_addr = public_ip::addr_v4()
        .await
        .expect("Could not find ipv4 addr");
    let ipv6_addr = public_ip::addr_v6()
        .await
        .expect("Could not find ipv6 addr");

    let api_key = args
        .api_key
        .or_else(|| env::var("OMGLOL_API_KEY").ok())
        .expect("Must pass in api_key arg or define OMGLOL_API_KEY environment variable");

    let client = Client::new();

    let dns_records = get_dns_records(&client, &api_key, &args.name)
        .await
        .expect("Failed to get existing DNS records");

    upsert_dns_record(
        &client,
        &api_key,
        &args.name,
        &dns_records,
        DnsRecordPayload {
            type_: DnsType::A,
            name: args.subdomain.clone(),
            data: ipv4_addr.to_string(),
        },
    )
    .await
    .expect("Failed to create A DNS record");

    upsert_dns_record(
        &client,
        &api_key,
        &args.name,
        &dns_records,
        DnsRecordPayload {
            type_: DnsType::AAAA,
            name: args.subdomain.clone(),
            data: ipv6_addr.to_string(),
        },
    )
    .await
    .expect("Failed to create AAAA DNS record");

    println!(
        "{}",
        format!(
            "Created/updated A (ipv4) DNS record at {}.{}.omg.lol pointing to {}",
            args.subdomain,
            args.name,
            ipv4_addr.to_owned()
        )
    );
    println!(
        "{}",
        format!(
            "Created/updated AAAA (ipv4) DNS record at {}.{}.omg.lol pointing to {}",
            args.subdomain,
            args.name,
            ipv6_addr.to_owned()
        )
    );
}

async fn upsert_dns_record(
    client: &Client,
    apikey: &String,
    name: &String,
    records: &Vec<DnsRecord>,
    payload: DnsRecordPayload,
) -> Result<(), Error> {
    let dns_name_qual = format!("{}.{}", payload.name, name);
    let opt_record = records
        .iter()
        .find(|v| v.type_ == payload.type_ && v.name == dns_name_qual);

    match opt_record {
        Some(record) => update_dns_record(client, apikey, name, record).await,
        None => create_dns_record(client, apikey, name, payload).await,
    }
}

#[derive(Deserialize)]
struct Response {
    response: DnsResponse,
}

#[derive(Deserialize)]
struct DnsResponse {
    dns: Vec<DnsRecord>,
}

async fn get_dns_records(
    client: &Client,
    apikey: &String,
    name: &String,
) -> Result<Vec<DnsRecord>, Error> {
    let response = client
        .get(format!("https://api.omg.lol/address/{}/dns", name))
        .header("Authorization", format!("Bearer {}", apikey))
        .send()
        .await?;
    let json = response.json::<Response>().await?;
    Ok(json.response.dns)
}

async fn create_dns_record(
    client: &Client,
    apikey: &String,
    name: &String,
    payload: DnsRecordPayload,
) -> Result<(), Error> {
    let response = client
        .post(format!("https://api.omg.lol/address/{}/dns", name))
        .header("Authorization", format!("Bearer {}", apikey))
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    let resp_test = response.text().await?;

    if status != 200 {
        println!("resp: {}", &resp_test);
        Err(resp_test)?
    } else {
        Ok(())
    }
}

async fn update_dns_record(
    client: &Client,
    apikey: &String,
    name: &String,
    payload: &DnsRecord,
) -> Result<(), Error> {
    let response = client
        .patch(format!(
            "https://api.omg.lol/address/{}/dns/{}",
            name, payload.id
        ))
        .header("Authorization", format!("Bearer {}", apikey))
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    let resp_test = response.text().await?;

    if status != 200 {
        println!("resp: {}", &resp_test);
        Err(resp_test)?
    } else {
        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
enum DnsType {
    A,
    AAAA,
    CAA,
    CNAME,
    MX,
    NS,
    SRV,
    TXT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DnsRecordPayload {
    #[serde(rename = "type")]
    type_: DnsType,
    name: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DnsRecord {
    id: i64,
    #[serde(rename = "type")]
    type_: DnsType,
    name: String,
    data: String,
}

#[derive(Debug)]
enum Error {
    Request(reqwest::Error),
    String(String),
}

impl From<reqwest::Error> for Error {
    fn from(s: reqwest::Error) -> Error {
        Error::Request(s)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::String(s)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Error {
        Error::String(s.to_owned())
    }
}
