use anyhow::{bail, Result};
use candid::TypeEnv;
use candid_parser::bindings::rust::{compile, Config};
use candid_parser::bindings::rust::Target;
use candid_parser::check_prog;
use ic_agent::export::Principal;
use ic_agent::hash_tree::LookupResult;
use ic_agent::identity::AnonymousIdentity;
use ic_agent::Agent;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;


const DEFAULT_FILENAME: &str = "canister/def.rs";

/// Enum of allowed canister types
#[derive(Debug, Clone, ValueEnum)]
enum TargetType {
    Agent,
    Canister,
}

/// Generate Rust type definitions for a canister on the fly
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The canister principal to fetch type definitions for
    #[arg(short, long, required = true)]
    canister: String,

    /// The output format for the type definitions
    #[arg(short, long, value_enum, default_value_t = TargetType::Agent)]
    target: TargetType,

    /// Path to store the generated types [default: canister/def.rs]
    #[arg(short, long)]
    path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();

    let canister = Principal::from_text(&args.canister)?;

    let path = match args.path {
        Some(ref p) => p.clone(),
        _ => {
            let default_path = PathBuf::from(DEFAULT_FILENAME);
            println!("Using default path: {:?}", default_path);
            default_path
        }
    };

    let target = match args.target {
        TargetType::Agent => Target::Agent,
        TargetType::Canister => Target::CanisterCall,
    };

    let agent = get_agent().await;
    let result = get_canister_public_def(agent, canister, target).await?;
    write_stream_to_file(path, result.clone());
    Ok(())
}

async fn get_canister_public_def(agent: Arc<Agent>, canister_id: Principal, target: Target) -> Result<String> {
    let candid_path = vec![
        b"canister",
        canister_id.as_slice(),
        b"metadata",
        b"candid:service",
    ];
    let tree = agent
        .read_state_raw(
            vec![candid_path.clone().into_iter().map(|v| v.into()).collect()],
            canister_id,
        )
        .await
        .map(|certificate| certificate.tree)?;

    if let LookupResult::Found(bytes) = tree.lookup_path(&candid_path) {
        let candid_description = String::from_utf8(bytes.to_vec())?;
        let mut env = TypeEnv::new();
        let mut config = Config::default();
        config.set_canister_id(canister_id);
        config.set_target(target);
        let ast = candid_description.parse()?;
        let actor = check_prog(&mut env, &ast)?;
        let res = compile(&config, &env, &actor);
        return Ok(res);
    }

    bail!("Unable to read state tree for canister {:?}", canister_id);
}

fn write_stream_to_file(name: PathBuf, data: String) {
    let mut file = File::create(name).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

async fn get_agent() -> Arc<Agent> {
    let agent = Arc::new(
        Agent::builder()
            .with_url("https://icp-api.io")
            .with_background_dynamic_routing()
            .with_identity(AnonymousIdentity)
            .build()
            .unwrap(),
    );

    agent.fetch_root_key().await.unwrap();
    agent
}
