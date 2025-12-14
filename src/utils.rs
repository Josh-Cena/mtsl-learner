use polars::prelude::*;
use std::collections::BTreeSet;

pub fn get_dict(path: &str) -> PolarsResult<DataFrame> {
    let schema = Schema::from_iter(vec![
        Field::new("word".into(), DataType::String),
        Field::new("probability".into(), DataType::Float64),
        Field::new("prob_silence".into(), DataType::Float64),
        Field::new("corr_pre_silence".into(), DataType::Float64),
        Field::new("corr_pre_non-silence".into(), DataType::Float64),
        Field::new("pronunciation".into(), DataType::String),
    ]);
    CsvReadOptions::default()
        .with_has_header(false)
        .with_parse_options(CsvParseOptions::default().with_separator(b'\t'))
        .with_schema(Some(schema.into()))
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()
}

pub fn project<'a>(input: &Vec<Vec<&'a str>>, tier: &BTreeSet<&'a str>) -> Vec<Vec<&'a str>> {
    input
        .iter()
        .map(|v| {
            v.iter()
                .filter(|symbol| {
                    **symbol == "START"
                        || **symbol == "END"
                        || tier.contains::<&str>(symbol)
                })
                .copied()
                .collect()
        })
        .collect()
}
