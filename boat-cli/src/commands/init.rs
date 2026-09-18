use anyhow::Result;

use crate::config::Configuration;

pub fn init() -> Result<()> {
    let config = Configuration::create_default()?;
    let toml = toml::to_string(&config)?;
    println!("{toml}");
    Ok(())
}
