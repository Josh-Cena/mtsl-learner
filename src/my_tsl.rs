use crate::utils::*;
use std::collections::BTreeSet;

fn is_useless(symb: &str, projections: &Vec<Vec<&str>>, alphabet: &Vec<&str>) -> bool {
    let mut bigrams: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut around_symb: BTreeSet<(&str, &str)> = BTreeSet::new();
    for proj in projections.iter() {
        for pair in proj.windows(2) {
            bigrams.insert((pair[0], pair[1]));
        }
        for i in 1..(proj.len() - 1) {
            if proj[i] == symb {
                around_symb.insert((proj[i - 1], proj[i + 1]));
            }
        }
    }
    if !bigrams.contains(&("START", symb)) {
        return false;
    }
    if !bigrams.contains(&(symb, "END")) {
        return false;
    }
    for symb2 in alphabet.iter() {
        if !bigrams.contains(&(symb, symb2)) {
            return false;
        }
        if !bigrams.contains(&(symb2, symb)) {
            return false;
        }
    }
    for pair in around_symb.iter() {
        if !bigrams.contains(pair) {
            return false;
        }
    }
    true
}

pub fn learn_tsl2_my<'a>(
    input: &Vec<Vec<&'a str>>,
    alphabet: &Vec<&'a str>,
) -> (BTreeSet<&'a str>, BTreeSet<(&'a str, &'a str)>) {
    let mut tier = BTreeSet::from_iter(alphabet.iter().copied());
    let mut projections = project(&input, &tier);
    for symbol in alphabet {
        if is_useless(symbol, &projections, alphabet) {
            tier.remove(symbol);
            projections = project(&projections, &tier);
        }
    }
    let mut grammar: BTreeSet<(&str, &str)> = BTreeSet::new();
    for proj in projections.iter() {
        for pair in proj.windows(2) {
            grammar.insert((pair[0], pair[1]));
        }
    }
    (tier, grammar)
}
