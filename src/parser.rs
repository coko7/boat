use anyhow::{Result, anyhow};
use log::debug;
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;

pub fn parse_csv_record<T>(input: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(input.as_bytes());
    let mut iter = rdr.deserialize();

    let record: T = iter.next().ok_or_else(|| anyhow!("no record found"))??;
    Ok(record)
}

pub fn parse_csv<T>(input: &str) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(input.as_bytes());
    let mut out = Vec::new();

    for result in rdr.deserialize() {
        let record: T = result?;
        debug!("{record:?}");
        out.push(record);
    }

    Ok(out)
}

pub fn deserialize_hashset<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(HashSet::new());
    }

    Ok(s.split('|').map(|v| v.trim().to_string()).collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        activity::{ActivityDefinition, ActivityLog},
        utils,
    };

    #[test]
    fn test_parse_activity_logs_csv() -> Result<()> {
        utils::init_test_logger();
        assert_eq!(
            vec![ActivityLog {
                id: "27e28d2ac61f35f7".to_string(),
                start: utils::parse_local_dt("2026-03-16 08:00:00")?,
                end: Some(utils::parse_local_dt("2026-03-16 09:30:00")?),
            }],
            parse_csv::<ActivityLog>(
                "id;start;end
27e28d2ac61f35f7;2026-03-16T08:00:00+01:00;2026-03-16T09:30:00+01:00"
            )?
        );
        Ok(())
    }

    #[test]
    fn test_parse_activity_definitions_csv() -> Result<()> {
        utils::init_test_logger();
        let mut tags = HashSet::new();
        tags.insert("code".to_string());
        tags.insert("cli".to_string());

        assert_eq!(
            vec![
                ActivityDefinition {
                    id: "1e702a61b5c30021".to_string(),
                    parent_id: None,
                    name: "take out trash".to_string(),
                    tags: HashSet::new(),
                },
                ActivityDefinition {
                    id: "979f13ada1031e05".to_string(),
                    parent_id: Some("2588c8f8ac72af67".to_string()),
                    name: "cook pasta".to_string(),
                    tags: HashSet::new(),
                },
                ActivityDefinition {
                    id: "bc7272ea7045d136".to_string(),
                    parent_id: Some("7a6e4e2b4b094661".to_string()),
                    name: "work on csv parser".to_string(),
                    tags
                }
            ],
            // TODO: handle duplicate activity IDss
            parse_csv::<ActivityDefinition>(
                "id;parent_id;name;tags
1e702a61b5c30021;;take out trash;
979f13ada1031e05;2588c8f8ac72af67;cook pasta;
bc7272ea7045d136;7a6e4e2b4b094661;work on csv parser;cli|code"
            )?
        );
        Ok(())
    }
}
