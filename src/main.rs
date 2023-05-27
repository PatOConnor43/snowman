use clap::{Parser, Subcommand};
use dialoguer::Editor;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use std::collections::HashMap;
use std::process::Command;

use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use std::env::var;
use std::error::Error;

use serde::{Deserialize, Serialize};

use config_file::FromConfigFile;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Environment {
    meta: EnvironmentMeta,
    model_id: String,
    data: EnvironmentData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EnvironmentData {
    //owner: String,
    team: Option<String>,
    #[serde(rename = "lastUpdatedBy")]
    last_updated_by: String,
    #[serde(rename = "lastRevision")]
    last_revision: i64,
    id: String,
    name: String,
    values: Vec<Value>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceMeta {
    model: String,
    action: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceMembers {
    users: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceDataState {
    is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceData {
    id: String,
    name: String,
    description: Option<String>,
    summary: String,
    #[serde(rename = "createdBy")]
    created_by: String,
    #[serde(rename = "updatedBy")]
    updated_by: String,
    team: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "visibilityStatus")]
    visibility_status: String,
    r#type: String,
    members: WorkspaceMembers,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    meta: WorkspaceMeta,
    model_id: String,
    data: WorkspaceData,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
struct Config {
    cookie: String,
    domain: String,
    environment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EnvironmentMeta {
    model: String,
    action: String,
    #[serde(rename = "forkedFrom")]
    forked_from: Option<ForkedFrom>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ForkedFrom {
    id: String,
    #[serde(rename = "forkName")]
    fork_name: String,
    name: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Value {
    key: String,
    value: String,
    enabled: bool,
    r#type: String,
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "snowman")]
#[command(about = "Bring Postman into your terminal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Activate a subshell with active environment variable")]
    Activate,

    #[command(about = "Print the current configuration to stdout")]
    Config,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    match args.command {
        Commands::Config => {
            let c = config()?;
            println!("cookie = \"{}\"\ndomain = \"{}\"", c.cookie, c.domain);
            Ok(())
        }
        Commands::Activate => {
            let c = config()?;
            let environment = activate(c)?;
            let environment_name = resolve_env_name(&environment);
            let values = environment.data.values;
            // Using this env var to determine what subshell to open probably isn't fullproof, but
            // good enough?
            let shell = var("SHELL").expect("Could not find active SHELL environment");
            let mut values_map = values
                .iter()
                .map(|v| (format!("SNOWMAN_{}", v.key), v.value.to_string()))
                .collect::<HashMap<String, String>>();
            values_map.insert("_SNOWMAN_ENVIRONMENT_NAME".to_string(), environment_name);
            let info_header = "The following environment variables have been set:";
            let mut info_body: Vec<String> = values_map.iter().map(|pair| format!("{}: {}", pair.0, pair.1)).collect();
            info_body.sort();
            println!("\n{}\n{}", info_header, info_body.join("\n"));
            Command::new(shell)
                .envs(values_map.clone())
                .status()
                .expect("Failed to create shell");
            Ok(())
        }
    }
}

fn activate(config: Config) -> Result<Environment, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        HeaderValue::from_str(&config.cookie).unwrap(),
    );

    let rest_client = reqwest::blocking::ClientBuilder::default()
        .default_headers(headers.clone())
        .build()?;
    let response: String = rest_client
        .get(format!("{}/_api/workspace", config.domain))
        .send()
        .unwrap()
        .text()
        .unwrap();
    let workspaces = serde_json::from_str::<Vec<Workspace>>(&response).unwrap();
    let selections = workspaces
        .iter()
        .map(|workspace| workspace.data.name.to_string())
        .collect::<Vec<String>>();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your workspace")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let workspace = &workspaces[selection];
    let response: String = rest_client
        .get(format!("{}/_api/environment", config.domain))
        .query(&[("workspace", &workspace.model_id)])
        .send()
        .unwrap()
        .text()
        .unwrap();
    let environments = serde_json::from_str::<Vec<Environment>>(&response).unwrap();
    let selections = environments
        .iter()
        .map(|e| resolve_env_name(e))
        .collect::<Vec<String>>();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your environment")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();

    Ok(environments[selection].clone())
}

fn resolve_env_name(environment: &Environment) -> String {
    match &environment.meta.forked_from {
        Some(forked_from) => format!("{} [{}]", forked_from.fork_name, forked_from.name),
        None => format!("{}", environment.data.name),
    }
}

fn config() -> Result<Config, Box<dyn Error>> {
    let config_directory = get_config_path();
    std::fs::create_dir_all(&config_directory)?;
    let config_file = format!("{}/snowman.toml", config_directory);
    let c = Config::from_config_file(&config_file);
    if let Err(_) = c {
        let content_result = Editor::new().edit(r"# Snowman Config
#                                           
# The value of your cookie should look something like: 'postman.sid=...'
cookie =

# The value of your domain is the url you use for your workspace. An example would be 'https://dark-trinity-5058.postman.co'
domain = 
");
        let content = content_result?;
        if content.is_none() {
            Err("Failed to write valid config")?
        }
        let content = content.unwrap();
        eprintln!("Writing to {}", config_file);
        std::fs::write(config_file.to_string(), content)?;
        return Ok(Config::from_config_file(&config_file)?)
    }
    Ok(c.unwrap().clone())
}

fn get_config_path() -> String {
    match var("XDG_CONFIG_HOME") {
        Ok(val) => format!("{}/snowman", val),
        Err(_) => {
            let home_path = home::home_dir().unwrap();
            format!("{}/.config/snowman", home_path.to_str().unwrap())
        }
    }
}
