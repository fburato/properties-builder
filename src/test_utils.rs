use std::collections::HashSet;
use std::fmt::Debug;

pub fn assert_contains_exactly_in_any_order<T: PartialEq + Debug, V: AsRef<Vec<T>>>(
    actual: V,
    expected: V,
) {
    let mut matched_indexes_in_actual: HashSet<usize> = HashSet::new();
    let mut unmatched_indexes_in_expected: HashSet<usize> = HashSet::new();

    for (i, e) in expected.as_ref().iter().enumerate() {
        let mut actual_index: Option<usize> = None;
        for (j, a) in actual.as_ref().iter().enumerate() {
            if matched_indexes_in_actual.contains(&j) {
                continue;
            }
            if e == a {
                matched_indexes_in_actual.replace(j);
                actual_index = Some(j);
                break;
            }
        }
        if actual_index.is_none() {
            unmatched_indexes_in_expected.replace(i);
        }
    }
    let mut excess: Vec<usize> = Vec::new();
    for (i, _) in actual.as_ref().iter().enumerate() {
        if !matched_indexes_in_actual.contains(&i) {
            excess.push(i);
        }
    }
    let mut result: String = "".to_string();
    let comma_separator = ", ";
    if !unmatched_indexes_in_expected.is_empty() {
        result = result + "The following elements where expected but not found:\n";
        let mut separator = "[";
        for i in unmatched_indexes_in_expected.iter() {
            result = result + separator + format!("{:?}", expected.as_ref()[*i]).as_str();
            separator = comma_separator;
        }
        result = result + "]\n"
    }
    if !excess.is_empty() {
        result = result + "The following elements where not expected:\n";
        let mut separator = "[";
        for a in excess {
            result = result + separator + format!("{:?}", actual.as_ref()[a]).as_str();
            separator = comma_separator;
        }
        result = result + "]";
    }
    if result != "" {
        panic!("{}", result);
    }
}
