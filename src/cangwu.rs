use super::{Cents, FactorElement};

pub fn limited_mappings(n_notes: FactorElement,
                    ek: Cents,
                    bmax: Cents,
                    plimit: &Vec<Cents>,
                    ) -> Vec<Vec<FactorElement>> {
    // Call things Cents but turn them to octaves/dimensinoless
    let ek = ek / 12e2;
    let bmax = bmax / 12e2;
    let plimit: Vec<Cents> = plimit.iter().cloned().map(|x| x/12e2).collect();
    let cap = bmax * bmax * (plimit.len() as Cents)
                            / (plimit[0] * plimit[0]);
    let epsilon2 = ek * ek / (1.0 + ek * ek);

    // mapping: the ET mapping with a new entry
    // tot: running total of w
    // tot2: running total of w squared
    fn more_limited_mappings(mapping: Vec<FactorElement>,
                             tot: Cents,
                             tot2: Cents,
                             cap: Cents,
                             epsilon2: Cents,
                             plimit: &Vec<Cents>,
                             ) -> Vec<Vec<FactorElement>> {
        let mut result = Vec::new();
        let i = mapping.len();
        let weighted_size = (mapping[i - 1] as Cents) / plimit[i - 1];
        let tot = tot + weighted_size;
        let tot2 = tot2 + weighted_size * weighted_size;
        let lambda = 1.0 - epsilon2;
        if i == plimit.len() {
            // recursion stops here
            result.push(mapping);
        }
        else {
            let toti = tot * lambda / ((i as Cents) + epsilon2);
            let error2 = tot2 - tot * toti;
            if error2 < cap {
                let target = plimit[i];
                let deficit: Cents = ((
                    (i + 1) as Cents * (cap - error2)
                    / (i as Cents + epsilon2)
                    ) as Cents).sqrt();
                let xmin = target * (toti - deficit);
                let xmax = target * (toti + deficit);
                for guess in intrange(xmin, xmax) {
                    let mut next_mapping = mapping.clone();
                    next_mapping.push(guess);
                    let results = more_limited_mappings(
                        next_mapping, tot, tot2,
                        cap, epsilon2, &plimit);
                    for new_result in results {
                        result.push(new_result);
                    }
                }
            }
        }
        result
    }
    more_limited_mappings(vec![n_notes], 0.0, 0.0, cap, epsilon2, &plimit)
}

fn intrange(x: Cents, y: Cents) -> std::ops::RangeInclusive<FactorElement> {
    ((x.ceil() as FactorElement) ..= (y.floor() as FactorElement))
}
