use std::collections::{BTreeMap, BTreeSet};

fn get_interveners<'a>(
    inputs: &Vec<Vec<&'a str>>,
    symb1: &'a str,
    symb2: &'a str,
) -> BTreeSet<BTreeSet<&'a str>> {
    let mut interveners: BTreeSet<BTreeSet<&'a str>> = BTreeSet::new();
    for word in inputs.iter() {
        for i in 0..word.len() {
            if word[i] != symb1 {
                continue;
            }
            let j = word[i + 1..]
                .iter()
                .position(|&s| s == symb2)
                .map(|p| p + i + 1);
            if let Some(j) = j {
                let intervening: BTreeSet<&'a str> = word[(i + 1)..j].iter().copied().collect();
                interveners.insert(intervening);
            }
        }
    }
    interveners
}

pub fn learn_mtsl2<'a>(
    input: &Vec<Vec<&'a str>>,
    alphabet: &Vec<&'a str>,
) -> BTreeMap<BTreeSet<&'a str>, BTreeSet<(&'a str, &'a str)>> {
    let input_bigrams: BTreeSet<(&str, &str)> = input
        .iter()
        .flat_map(|word| {
            word.windows(2)
                .map(|pair| (pair[0], pair[1]))
                .collect::<Vec<(&str, &str)>>()
        })
        .collect();
    let mut grammars: BTreeMap<BTreeSet<&str>, BTreeSet<(&str, &str)>> = BTreeMap::new();
    for symb1 in std::iter::once("START").chain(alphabet.clone()) {
        for symb2 in std::iter::once("END").chain(alphabet.clone()) {
            if input_bigrams.contains(&(symb1, symb2)) {
                continue;
            }
            // Not attested bigram
            let mut tier: BTreeSet<&str> = alphabet.iter().copied().collect();
            let interveners = get_interveners(&input, symb1, symb2);
            for symb in alphabet.iter() {
                if *symb == symb1 || *symb == symb2 {
                    continue;
                }
                if interveners.iter().all(|x| {
                    !x.contains(symb)
                        || interveners
                            .contains(&x.difference(&BTreeSet::from([*symb])).cloned().collect())
                }) {
                    tier.remove(symb);
                }
            }
            grammars
                .entry(tier.clone())
                .or_insert_with(BTreeSet::new)
                .insert((symb1, symb2));
        }
    }
    grammars
}
