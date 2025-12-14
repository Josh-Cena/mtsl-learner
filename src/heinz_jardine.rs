use polars::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

// symbol1 => symbol2 => set of intervener sets
pub type Paths<'a> = BTreeMap<&'a str, BTreeMap<&'a str, BTreeSet<BTreeSet<&'a str>>>>;

pub fn paths2<'a>(words: &'a ChunkedArray<StringType>) -> Paths<'a> {
    let mut paths: Paths<'a> = BTreeMap::new();
    for word in words.into_no_null_iter() {
        let symbols: Vec<&str> = std::iter::once("START")
            .chain(word.split_whitespace())
            .chain(std::iter::once("END"))
            .collect();
        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                let start = symbols[i];
                let end = symbols[j];
                let interveners: BTreeSet<&str> = symbols[(i + 1)..j].iter().copied().collect();
                paths
                    .entry(start)
                    .or_insert_with(BTreeMap::new)
                    .entry(end)
                    .or_insert_with(BTreeSet::new)
                    .insert(interveners);
            }
        }
    }
    paths
}

fn get_tier<'a>(
    tier: BTreeSet<&'a str>,
    paths: Paths<'a>,
    i_init: usize,
    alphabet: &Vec<&'a str>,
) -> (BTreeSet<&'a str>, Paths<'a>) {
    // PTi = {〈τ1, X, τ2〉 ∈ P |τ1, τ2 ∈ Ti ∪ {START, END}}
    let mut tier_paths: Paths = BTreeMap::new();
    for (start, targets) in paths.iter() {
        if !tier.contains(start) && *start != "START" {
            continue;
        }
        let mut new_targets: BTreeMap<&str, BTreeSet<BTreeSet<&str>>> = BTreeMap::new();
        for (end, interveners_sets) in targets.iter() {
            if !tier.contains(end) && *end != "END" {
                continue;
            }
            new_targets.insert(*end, interveners_sets.clone());
        }
        tier_paths.insert(*start, new_targets);
    }
    // Hi = Σ − Ti
    let non_tier = alphabet
        .iter()
        .copied()
        .collect::<BTreeSet<&str>>()
        .difference(&tier)
        .cloned()
        .collect::<BTreeSet<_>>();
    'next_i: for i in i_init..alphabet.len() as usize {
        let symb = alphabet[i];
        // Condition 1: ∃X ⊆ Hi such that 〈START, X, σ〉∈ P_Ti
        let start_to_symb = tier_paths.get("START").and_then(|m| m.get(symb));
        // Inverse: ∀〈START, X, σ〉∈ P_Ti, X /⊆ Hi
        if start_to_symb.is_none_or(|x| {
            x.iter()
                .all(|interveners| !interveners.is_subset(&non_tier))
        }) {
            continue 'next_i;
        }
        // ∃X′ ⊆ Hi such that 〈σ, X′, END〉 ∈ P_Ti
        let symb_to_end = tier_paths.get(symb).and_then(|m| m.get("END"));
        // Inverse: ∀〈σ, X′, END〉 ∈ P_Ti, X′ /⊆ Hi
        if symb_to_end.is_none_or(|x| {
            x.iter()
                .all(|interveners| !interveners.is_subset(&non_tier))
        }) {
            continue 'next_i;
        }
        // ∀σ′ ∈ Ti, ∃Y ⊆ Hi such that 〈σ, Y, σ′〉 ∈ P_Ti
        // Inverse: ∃σ′ ∈ Ti such that ∀〈σ, Y, σ′〉 ∈ P_Ti, Y /⊆ Hi
        if tier.iter().any(|other| {
            let symb_to_other = tier_paths.get(symb).and_then(|m| m.get(other));
            symb_to_other.is_none_or(|x| {
                x.iter()
                    .all(|interveners| !interveners.is_subset(&non_tier))
            })
        }) {
            continue 'next_i;
        }
        // ∀σ′ ∈ Ti, ∃Y′ ⊆ Hi such that 〈σ′, Y′, σ〉 ∈ P_Ti
        // Inverse: ∃σ′ ∈ Ti such that ∀〈σ′, Y′, σ〉 ∈ P_Ti, Y′ /⊆ Hi
        if tier.iter().any(|other| {
            let other_to_symb = tier_paths.get(other).and_then(|m| m.get(symb));
            other_to_symb.is_none_or(|x| {
                x.iter()
                    .all(|interveners| !interveners.is_subset(&non_tier))
            })
        }) {
            continue 'next_i;
        }
        // for each 〈τ1, Z, τ2〉 ∈ PTi with
        // * τ1, τ2 ∈ ((Ti ∪ {START, END}) − {σ})
        // * σ ∈ Z
        // * Z − {σ} ⊆ Hi
        // ==> ∃Z′ ⊆ Hi such that 〈τ1, Z′, τ2〉 ∈ P_Ti
        for symb1 in tier.iter().chain(std::iter::once(&"START")) {
            if symb1 == &symb {
                continue;
            }
            for symb2 in tier.iter().chain(std::iter::once(&"END")) {
                if symb2 == &symb {
                    continue;
                }
                let symb1_to_symb2 = tier_paths.get(symb1).and_then(|m| m.get(symb2));
                // No such path; precondition vacuously satisfied
                if symb1_to_symb2.is_none() {
                    continue;
                }
                let symb1_to_symb2 = symb1_to_symb2.unwrap();
                // No such path; precondition vacuously satisfied
                if !symb1_to_symb2.iter().any(|interveners| {
                    interveners.contains(symb)
                        && interveners.is_subset(
                            &non_tier
                                .union(&BTreeSet::from([symb]))
                                .cloned()
                                .collect::<BTreeSet<&str>>(),
                        )
                }) {
                    continue;
                }
                // Check postcondition
                if symb1_to_symb2
                    .iter()
                    .all(|interveners| !interveners.is_subset(&non_tier))
                {
                    // Postcondition fails; proceed to next i
                    continue 'next_i;
                }
            }
        }
        // If we reach here, symbol is free and exclusive blocker
        return get_tier(
            tier.difference(&BTreeSet::from([symb]))
                .cloned()
                .collect::<BTreeSet<&str>>(),
            tier_paths,
            i + 1,
            alphabet,
        );
    }
    (tier, tier_paths)
}

pub fn learn_tsl2_heinz<'a>(
    input: &'a ChunkedArray<StringType>,
    alphabet: &Vec<&'a str>,
) -> (BTreeSet<&'a str>, BTreeSet<(&'a str, &'a str)>) {
    let paths = paths2(&input);
    let (tier, tier_paths) = get_tier(alphabet.clone().into_iter().collect(), paths, 0, &alphabet);
    let non_tier = alphabet
        .iter()
        .copied()
        .collect::<BTreeSet<&str>>()
        .difference(&tier)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut grammar: BTreeSet<(&str, &str)> = BTreeSet::new();
    for (symb1, symb1_map) in tier_paths.iter() {
        for (symb2, interveners_sets) in symb1_map.iter() {
            for interveners in interveners_sets.iter() {
                if interveners.is_subset(&non_tier) {
                    grammar.insert((*symb1, *symb2));
                }
            }
        }
    }
    (tier, grammar)
}
