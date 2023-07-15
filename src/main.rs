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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentResponse {
    pub environment: Environment,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub created_at: String,
    pub updated_at: String,
    pub values: Vec<Value>,
    pub is_public: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDigestsResponse {
    pub environments: Vec<EnvironmentDigest>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDigest {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub owner: String,
    pub uid: String,
    pub is_public: bool,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesResponse {
    pub workspaces: Vec<Workspace>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub visibility: String,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
struct Config {
    apikey: String,
}

#[derive(Debug, Parser)]
#[command(name = "snowman")]
#[command(about = "Bring Postman into your terminal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Activate a sub-shell with active environment variable")]
    Activate,

    #[command(about = "Print the current configuration to stdout")]
    Config,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    match args.command {
        Commands::Config => {
            let c = config()?;
            println!("apikey = \"{}\"", c.apikey);
            Ok(())
        }
        Commands::Activate => {
            let c = config()?;
            let environment = activate(c)?;
            let environment_name = environment.name;
            let values = environment.values;
            // Using this env var to determine what subshell to open probably isn't fullproof, but
            // good enough?
            let shell = var("SHELL").expect("Could not find active SHELL environment");
            let mut values_map =
                values
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
        "X-Api-Key",
        HeaderValue::from_str(&config.apikey).unwrap(),
    );

    let rest_client = reqwest::blocking::ClientBuilder::default()
        .default_headers(headers.clone())
        .build()?;
    let response: String = rest_client
        .get(format!("https://api.getpostman.com/workspaces"))
        .send()
        .unwrap()
        .text()
        .unwrap();
    dbg!(&response);
    let response = serde_json::from_str::<WorkspacesResponse>(&response).unwrap();
    let selections = response.workspaces
        .iter()
        .map(|workspace| workspace.name.to_string())
        .collect::<Vec<String>>();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your workspace")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let workspace = &response.workspaces[selection];
    let response: String = rest_client
        .get(format!("https://api.getpostman.com/environments"))
        .query(&[("workspace", &workspace.id)])
        .send()
        .unwrap()
        .text()
        .unwrap();
    let response = serde_json::from_str::<EnvironmentDigestsResponse>(&response).unwrap();
    let selections = response.environments
        .iter()
        .map(|e| e.name.to_string())
        .collect::<Vec<String>>();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your environment")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let response: String = rest_client
        .get(format!("https://api.getpostman.com/environments/{}", response.environments[selection].id))
        .send()
        .unwrap()
        .text()
        .unwrap();
    let response = serde_json::from_str::<EnvironmentResponse>(&response).unwrap();

    Ok(response.environment)
}

fn config() -> Result<Config, Box<dyn Error>> {
    let config_directory = get_config_path();
    std::fs::create_dir_all(&config_directory)?;
    let config_file = format!("{}/snowman.toml", config_directory);
    let c = Config::from_config_file(&config_file);
    if let Err(_) = c {
        let content_result = Editor::new().edit(r"# Snowman Config
#
# The value of your public API token should look like: 'PMAK-xxxxxxxxxxxxxxxxxxxxxxxx-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'.
# You can make an API key at https://<domain>/settings/me/api-keys
apikey =
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
