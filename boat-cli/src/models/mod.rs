use anyhow::{Result, anyhow};
use tabular::{Row, Table};
use yansi::Paint;

pub mod activity;
pub mod activity_log;
pub mod boat_data;
pub mod log;
pub mod tag;

/// A single displayable column: (key used in --fields/config, header label, alignment)
pub type FieldSpec = (&'static str, &'static str, char);

pub trait TablePrintable {
    /// Renders as a table. `fields` selects and orders columns by key (see `RowPrintable::field_specs`).
    /// `None` uses every column in its default order.
    fn to_printable_table(&self, fields: Option<&[String]>) -> Result<Table>;
}

pub trait RowPrintable {
    /// Every available column, in default display order.
    fn field_specs() -> Vec<FieldSpec>;
    /// Values in the same order as `field_specs()`.
    fn row_values(&self) -> Vec<String>;
    fn style_cell(&self, value: String) -> String {
        value
    }
}

fn resolve_field_indices(specs: &[FieldSpec], fields: Option<&[String]>) -> Result<Vec<usize>> {
    let Some(keys) = fields else {
        return Ok((0..specs.len()).collect());
    };

    keys.iter()
        .map(|key| {
            specs
                .iter()
                .position(|(field_key, _, _)| field_key.eq_ignore_ascii_case(key))
                .ok_or_else(|| {
                    let available = specs
                        .iter()
                        .map(|(k, _, _)| *k)
                        .collect::<Vec<_>>()
                        .join(", ");
                    anyhow!("unknown field '{key}' (available: {available})")
                })
        })
        .collect()
}

impl<T> TablePrintable for Vec<T>
where
    T: RowPrintable,
{
    fn to_printable_table(&self, fields: Option<&[String]>) -> Result<Table> {
        let specs = T::field_specs();
        let indices = resolve_field_indices(&specs, fields)?;

        let row_spec = indices
            .iter()
            .map(|&i| format!("{{:{}}}", specs[i].2))
            .collect::<Vec<_>>()
            .join("  ");
        let mut table = Table::new(&row_spec);

        let mut header = Row::new();
        for &i in &indices {
            header.add_ansi_cell(Paint::new(specs[i].1).underline().to_string());
        }
        table.add_row(header);

        for item in self.iter() {
            let values = item.row_values();
            let mut row = Row::new();
            for &i in &indices {
                let styled = item.style_cell(values[i].clone());
                row.add_ansi_cell(styled);
            }
            table.add_row(row);
        }

        Ok(table)
    }
}
