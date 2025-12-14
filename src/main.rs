mod heinz_jardine;
mod mcmullin;
mod my_tsl;
mod utils;

use heinz_jardine::*;
use itertools::MultiUnzip;
use itertools::iproduct;
use mcmullin::*;
use my_tsl::*;
use polars::prelude::*;
use std::collections::BTreeSet;
use std::env;
use std::fs::File;
use utils::*;

fn main() -> PolarsResult<()> {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let learner_type = &args[2];
    let df = get_dict(format!("dictionaries/{path}.dict").as_str())?;
    let input = df.column("pronunciation")?.as_series().unwrap().str()?;
    let mut alphabet_set: BTreeSet<&str> = BTreeSet::new();
    let input = input.filter(&input.not_equal("spn"))?;
    let input: Vec<Vec<&str>> = input
        .into_no_null_iter()
        .map(|word| {
            let symbols = word.split_whitespace();
            alphabet_set.extend(symbols.clone());
            std::iter::once("START")
                .chain(symbols)
                .chain(std::iter::once("END"))
                .collect()
        })
        .collect();
    let alphabet: Vec<&str> = alphabet_set.iter().copied().collect();
    if learner_type == "tsl" {
        let (tier, grammar) = learn_tsl2_my(&input, &alphabet);
        let mut tier_df = df!(
            "symbol" => alphabet.clone(),
            "included" => alphabet.iter().map(|x| tier.contains(x)).collect::<Vec<bool>>(),
        )?;
        CsvWriter::new(File::create(format!("grammars-tsl/{path}-tier.csv"))?)
            .finish(&mut tier_df)?;
        let grammar_data: (Vec<&str>, Vec<&str>, Vec<bool>) = iproduct!(
            alphabet.iter().chain(std::iter::once(&"START")),
            alphabet.iter().chain(std::iter::once(&"END")),
        )
        .map(|(s1, s2)| (s1, s2, grammar.contains(&(*s1, *s2))))
        .multiunzip();
        let mut grammar_df = df!(
            "symb1" => grammar_data.0,
            "symb2" => grammar_data.1,
            "allowed" => grammar_data.2,
        )?;
        CsvWriter::new(File::create(format!("grammars-tsl/{path}-grammar.csv"))?)
            .finish(&mut grammar_df)?;
    } else {
        let grammars = learn_mtsl2(&input, &alphabet);
        let grammar_data: (Vec<u32>, Vec<String>, Vec<u32>, Vec<String>) = grammars
            .iter()
            .map(|(tier, grammar)| {
                let tier_str = format!("{:?}", tier);
                let forbidden_str = format!("{:?}", grammar);
                (
                    tier.len() as u32,
                    tier_str,
                    grammar.len() as u32,
                    forbidden_str,
                )
            })
            .multiunzip();
        let mut grammar_df = df!(
            "tier_size" => grammar_data.0,
            "tier" => grammar_data.1,
            "forbidden_size" => grammar_data.2,
            "forbidden" => grammar_data.3,
        )?;
        CsvWriter::new(File::create(format!("grammars-mtsl/{path}-grammars.csv"))?)
            .finish(&mut grammar_df)?;
    }
    Ok(())
}
