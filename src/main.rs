mod vault;
use crate::vault::VaultClient;

use std::env;
use clap::{ Arg, Command };
use regex::Regex;


fn main() {
    let pkg_version: &str = env!("CARGO_PKG_VERSION");
    let matches: clap::ArgMatches = Command::new("Vault To Terraform")
    .version(pkg_version)
    .author("Andrey Chushkin, andrey@chushkin.tech")
    .about("Supplying Vault secrets to Terraform environment variables")
    .arg(Arg::new("addr").long("vault-address").help("Vault address").value_name("ADDR"))
    .arg(Arg::new("token").long("vault-token").help("Vault token").value_name("TOKEN"))
    .arg(Arg::new("prefix").long("value-prefix").help("Prefixes for output").value_name("PREFIX").default_value("TF_VAR_"))
    .arg(Arg::new("mount").help("Secret mount point").index(1).required(true).value_name("MOUNT_POINT"))
    .arg(Arg::new("path").help("Vault secret path").index(2).required(true).value_name("PATH"))
    .get_matches();
    
    let env_vault_addr: Result<String, env::VarError> = std::env::var("VAULT_ADDR");
    let arg_vault_addr: Option<&String> = matches.get_one::<String>("addr");

    let env_vault_token: Result<String, env::VarError> = std::env::var("VAULT_TOKEN");    
    let arg_vault_token: Option<&String> = matches.get_one::<String>("token");
    
    let vault_addr: String = validate_address(env_vault_addr, arg_vault_addr).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    
    let vault_token: String;
    if let Ok(t) = env_vault_token {
        vault_token = t;
    } else if let Some(t) = arg_vault_token {
        vault_token = t.to_string();
    } else {
        eprintln!("Vault token not provided");
        std::process::exit(1);
    }

    let vault = VaultClient::new(vault_addr, vault_token).unwrap_or_else(|e| {
        eprintln!("{}", VaultClient::err(e));
        std::process::exit(1);
    });

    let sealed = vault.is_sealed().unwrap_or_else(|e| {
        eprintln!("{}", VaultClient::err(e));
        std::process::exit(1);
    });
    if sealed {
        eprintln!("Vault is sealed, cannot continue");
        std::process::exit(1);
    }

    let secret_path = matches.get_one::<String>("path").unwrap();
    let mount_path = matches.get_one::<String>("mount").unwrap();
    
    let version: vault::VaultVersion;
    let sv = vault.get_secret_version(mount_path.to_string());
    if let Ok(v) = sv  {
        version = v;
    } else {
        if let Err(e) = sv {
            eprintln!("{}", VaultClient::err(e));
        } else {
            eprintln!("Invalid vault response getting KV version");
        }
        std::process::exit(1);
    } 

    let r = vault.get_secret(version, mount_path.to_string(), secret_path.to_string());
    let secrets = r.unwrap_or_else(|e| {
        eprintln!("{}", VaultClient::err(e));
        std::process::exit(1);
    });

    let prefix = matches.get_one::<String>("prefix").unwrap();
    for (k, v) in secrets {
        println!("export {}{}=$\'{}\'", prefix.to_uppercase(), k, escape_value(v));
    }
}

fn validate_address(env_addr: Result<String, env::VarError>, arg_addr: Option<&String>) -> Result<String, String> {
    let addr: String;
    if let Ok(a) = env_addr {
        addr = a;
    } else if let Some(a) = arg_addr {
        addr = a.to_string();
    } else {
        return Err(String::from("No vault address provided"));
    }

    let re = Regex::new(r"^http(s?)://.+").unwrap();
    if re.is_match(addr.as_str()) {
        Ok(addr)
    } else {
        Err(String::from("Invalid vault address provided"))
    }
}

fn escape_value(source: String) -> String {
    let re = Regex::new(r#"[!$\"\'\n]"#).unwrap();
    let target = re.replace_all(&source, |caps: &regex::Captures| {
        match &caps[0] {
            "!" => r#"\!"#,
            "$" => r#"\$"#,
            "\"" => r#"\""#,
            "\'" => r#"\'"#,
            "\n" => r#"\n"#,
            _ => &caps[0]
        }.to_string()
    });
    target.to_string()
}

