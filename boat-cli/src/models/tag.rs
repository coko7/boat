use boat_lib::models::tag::Tag as DatabaseTag;
use boat_lib::repository::Id;
use serde::{Deserialize, Serialize};

use crate::models::{FieldSpec, RowPrintable};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintableTag {
    pub id: Id,
    pub name: String,
}

impl PrintableTag {
    pub fn from_tag(tag: &DatabaseTag) -> Self {
        Self {
            id: tag.id,
            name: tag.name.clone(),
        }
    }
}

impl RowPrintable for PrintableTag {
    fn field_specs() -> Vec<FieldSpec> {
        vec![("id", "ID", '>'), ("name", "Name", '<')]
    }

    fn row_values(&self) -> Vec<String> {
        vec![self.id.to_string(), self.name.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_from_tag_works() {
        let db_tag = DatabaseTag {
            id: 1,
            name: "foo".to_string(),
        };
        let tag = PrintableTag::from_tag(&db_tag);
        assert_eq!(tag.id, 1);
        assert_eq!(tag.name, "foo");
    }
}
