use lazy_static::lazy_static;
use std::collections::HashMap;

use super::ETMap;

lazy_static! {
    pub static ref NAMES_BY_LIMIT: HashMap<Vec<&'static str>, HashMap<ETMap, &'static str>> =
        HashMap::from([
            (
                vec!["2", "5", "11"],
                HashMap::from([
                    (vec![1, 3, 2, 0, -7], "Wizz"),
                    (vec![1, 1, 0, 1, 0, 0], "2.5.11-limit JI"),
                ]),
            ),
            (
                vec!["2", "3", "5"],
                HashMap::from([(vec![1, 4, 1, 0, -4], "Meantone"),]),
            ),
        ]);
}
