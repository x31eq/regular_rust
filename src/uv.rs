extern crate nalgebra as na;
use na::DMatrix;

use super::cangwu::filtered_equal_temperaments;
use super::{
    Cents, ETMap, ETSlice, Exponent, Mapping, echelon_form,
    hermite_normal_form, normalize_positive,
};

/// Return the commatic unison vector for a mapping with
/// only one dimension short
pub fn only_unison_vector(mapping: &Mapping) -> Option<ETMap> {
    let rank = mapping.len();
    if rank == 0 {
        return None;
    }
    let dimension = mapping[0].len();
    if rank + 1 != dimension {
        return None;
    }
    let fmap = float_matrix_from_mapping(mapping);
    let mut sq = fmap.transpose().insert_column(0, 0.0);
    for i in 0..dimension {
        sq[(i, 0)] = 1.0;
        if let Some(inverse) = sq.clone().try_inverse() {
            let adjoint = inverse * sq.clone().determinant();
            return mapping_from_float_matrix(adjoint).into_iter().next();
        }
        sq[(i, 0)] = 0.0;
    }
    None
}

pub fn get_ets_tempering_out(
    plimit: &[Cents],
    ek: Cents,
    unison_vectors: &[ETMap],
    n_results: usize,
) -> Mapping {
    filtered_equal_temperaments(
        plimit,
        |et| tempers_out(unison_vectors, et),
        ek,
        n_results,
    )
}

fn tempers_out(mapping: &[ETMap], interval: &ETSlice) -> bool {
    mapping.iter().all(|et| dotprod(et, interval) == 0)
}

/// Choose a value for the cangwu badness parameter
/// for a search based on these unison vectors.
/// This is a rough guess that has to be precise for
/// backwards compatibility reasons
pub fn ek_for_search(limit: &[Cents], uvs: &[ETMap]) -> Cents {
    uvs.iter()
        .map(|uv| inherent_error(limit, uv))
        .fold(0.0, Cents::max)
}

fn inherent_error(limit: &[Cents], uv: &ETSlice) -> Cents {
    if uv.is_empty() {
        // senseless question, return something to avoid panics
        return 0.0;
    }
    let q = limit
        .iter()
        .zip(uv.iter())
        .map(|(&x, &y)| x / 12e2 * y as Cents);
    let len = limit.len() as Cents;
    let mean = q.clone().sum::<Cents>() / len;
    let rms = (q.map(|x| x * x).sum::<Cents>() / len).sqrt();
    // Calculate in octaves for consistency with Python but return cents
    (mean / rms).abs() * 12e2
}

fn dotprod(a: &[Exponent], b: &[Exponent]) -> i64 {
    a.iter()
        .zip(b.iter())
        // multiply as i64 to avoid overflows
        .map(|(&m, &n)| (m as i64) * (n as i64))
        .sum()
}

/// Get unison vectors from a mapping and TLL-reduce them
pub fn unison_vector_basis(plimit: &[Cents], mapping: &[ETMap]) -> Mapping {
    rtlll(plimit, &saturated_kernel_basis(mapping))
        .into_iter()
        .map(|uv| normalize_positive(plimit, uv))
        .collect()
}

pub fn saturated_kernel_basis(vectors: &[ETMap]) -> Mapping {
    saturate(&kernel_basis(vectors))
        .expect("calculated basis not of full rank")
}

/// Get unison vectors from a mapping, or vice versa.
/// Results aren't simple and might introduce torsion.
fn kernel_basis(vectors: &[ETMap]) -> Mapping {
    // The algorithm originally came from
    // http://en.wikipedia.org/wiki/Null_space#Basis
    // but they kept taking it away because it isn't efficient.
    // But it is easy to implement.

    if vectors.is_empty() {
        return vec![];
    }
    let n_rows = vectors.len();
    let mut prepared = transpose(vectors);
    let n_cols = prepared.len();
    for (i, v) in prepared.iter_mut().enumerate() {
        for j in 0..n_cols {
            v.push(if i == j { 1 } else { 0 });
        }
    }
    echelon_form(&prepared)
        .into_iter()
        .filter_map(|v| {
            debug_assert_eq!(n_rows + n_cols, v.len());
            if v[..n_rows].iter().all(|&x| x == 0)
                && !v.iter().all(|&x| x == 0)
            {
                Some(v[n_rows..].to_vec())
            } else {
                None
            }
        })
        .collect()
}

/// Remove torsion from a basis.
/// Returns None when the vectors are not of full rank.
fn saturate(vectors: &[ETMap]) -> Option<Mapping> {
    // c.f. http://www.wstein.org/papers/hnf/
    // pernet-stein-fast_computation_of_hnf_of_random_integer_matrices.pdf
    if vectors.is_empty() {
        return Some(vec![]);
    }
    debug_assert!(vectors.iter().all(|row| row.len() == vectors[0].len()));
    debug_assert_ne!(vectors[0], vec![]);

    let n_vecs = vectors.len();
    let hermite = hermite_normal_form(vectors);
    debug_assert!(hermite.iter().all(|row| row.len() == vectors[0].len()));
    debug_assert_eq!(hermite.len(), n_vecs);

    let mut double_hermite = hermite_normal_form(&transpose(&hermite));
    debug_assert!(
        double_hermite
            .iter()
            .skip(n_vecs)
            .all(|row| row.iter().all(|&x| x == 0))
    );
    if n_vecs == 1 {
        let gcd = double_hermite[0][0];
        (gcd != 0).then_some(())?;
        return Some(vec![vectors[0].iter().map(|x| x / gcd).collect()]);
    }
    double_hermite.drain(n_vecs..);
    let double_hermite = float_matrix_from_mapping(&double_hermite);
    debug_assert_eq!(double_hermite.shape(), (n_vecs, n_vecs));

    let transformation = double_hermite.try_inverse()?;
    let hermite_matrix = float_matrix_from_mapping(&hermite);
    let result = transformation.transpose() * hermite_matrix;
    Some(mapping_from_float_matrix(result))
}

fn transpose<T: Clone>(m: &[Vec<T>]) -> Vec<Vec<T>> {
    if m.is_empty() {
        vec![vec![]]
    } else {
        debug_assert!(m.iter().all(|row| row.len() == m[0].len()));
        (0..m[0].len())
            .map(|i| m.iter().map(|row| row[i].clone()).collect())
            .collect()
    }
}

fn float_matrix_from_mapping(m: &Mapping) -> DMatrix<f64> {
    let n_cols = m.first().map_or(0, Vec::len);
    debug_assert!(m.iter().all(|row| row.len() == n_cols));
    DMatrix::from_row_iterator(
        m.len(),
        n_cols,
        m.iter().flat_map(|m| m.iter()).map(|&x| x as f64),
    )
}

fn mapping_from_float_matrix(m: DMatrix<f64>) -> Mapping {
    m.row_iter()
        .map(|row| row.iter().map(|&x| x.round() as Exponent).collect())
        .collect()
}

/// Recursive Tenney-weighted LLL
pub fn rtlll(plimit: &[Cents], vectors: &[ETMap]) -> Mapping {
    if vectors.len() < 2 {
        return vectors.to_vec();
    }
    let mut lll = tlll(plimit, vectors);
    let mut result = vec![lll.swap_remove(0)];
    result.append(&mut rtlll(plimit, &lll));
    result
}

/// Tenney-weighted LLL
pub fn tlll(plimit: &[Cents], vectors: &[ETMap]) -> Mapping {
    LLLReducer::new(plimit).reduce(vectors)
}

/// The book sets this it 2.  Lower numbers mean closer convergence.
/// I think it works from 2 to 4 but I'm not sure.
/// c.f. https://math.mit.edu/~apost/courses/18.204-2016/18.204_Xinyue_Deng_final_paper.pdf)
/// Testing shows it does work lower than 2, however
const LLL_TERMINATION_CONSTRAINT: f64 = 1.3;

/// LLL reduction with a Euclidean inner product
/// Based on Modern Computer Algebra,
/// von zur Gathen & Gerhard, p.449
struct LLLReducer {
    weights: Vec<f64>,
}

impl LLLReducer {
    pub fn new(plimit: &[Cents]) -> Self {
        LLLReducer {
            weights: plimit.iter().map(|x| x * x).collect(),
        }
    }

    pub fn reduce(&self, vectors: &[ETMap]) -> Mapping {
        let n = vectors.len();
        if n == 0 {
            return vec![];
        }
        debug_assert!(
            vectors.iter().all(|row| row.len() == vectors[0].len()),
        );
        let mut g = vectors.to_vec();
        let (mut gs, mut m) = self.gram_schmidt_orthogonalization(&g);
        let mut i = 1;
        while i < n {
            for j in (0..i).rev() {
                g[i] = g[i]
                    .iter()
                    .zip(&g[j])
                    .map(|(&gi, &gj)| gi - round_lll(m[i][j]) * gj)
                    .collect();
                (gs, m) = self.gram_schmidt_orthogonalization(&g);
            }
            if i > 0
                && self.prod(&gs[i - 1], &gs[i - 1])
                    > LLL_TERMINATION_CONSTRAINT * self.prod(&gs[i], &gs[i])
            {
                g.swap(i - 1, i);
                (gs, m) = self.gram_schmidt_orthogonalization(&g);
                i -= 1;
            } else {
                i += 1;
            }
        }
        g
    }

    fn gram_schmidt_orthogonalization(
        &self,
        basis: &[ETMap],
    ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        debug_assert!(
            basis.iter().all(|row| row.len() == self.weights.len()),
        );
        let mut m = vec![];
        let mut gramian: Vec<Vec<f64>> = vec![];
        let mut idrow_orig = vec![1.0];
        let idrow = &mut idrow_orig;
        for _ in 0..basis.len() {
            idrow.push(0.0);
        }
        for row in basis {
            let frow: Vec<f64> = row.iter().map(|&x| x as f64).collect();
            idrow.pop();
            let row: Vec<f64> = gramian
                .iter()
                .map(|g| self.prod(&frow, g) / self.prod(g, g))
                .chain(idrow.clone())
                .collect();
            let mut new_row = frow.clone();
            for (mcell, grow) in row.iter().zip(gramian.iter()) {
                new_row = frow
                    .iter()
                    .zip(grow.iter())
                    .map(|(&x, &g)| x - mcell * g)
                    .collect();
            }
            gramian.push(new_row);
            m.push(row);
        }
        debug_assert!(
            gramian.iter().all(|row| row.len() == self.weights.len()),
        );
        (gramian, m)
    }

    /// Inner product implementation
    fn prod(&self, u: &[f64], v: &[f64]) -> f64 {
        self.weights
            .iter()
            .zip(u.iter())
            .zip(v.iter())
            .map(|((&x, &m), &n)| x * m * n)
            .sum()
    }
}

/// Rounding function the book I copied from specifies
fn round_lll(x: f64) -> Exponent {
    (x + 0.5).next_down().floor() as Exponent
}

#[test]
fn meantone5() {
    let mapping = vec![vec![12, 19, 28], vec![7, 11, 16]];
    let expected = vec![-4, 4, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[4, -1, -1]));
    assert!(!tempers_out(&mapping, &[-3, -1, 1]));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv = normalize_positive(&super::PrimeLimit::new(5).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn marvel7() {
    let mapping = vec![
        vec![41, 65, 95, 115],
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
    ];
    let expected = vec![-5, 2, 2, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[-4, 4, -1, 0]));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv = normalize_positive(&super::PrimeLimit::new(7).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn marvel7_reordered() {
    let mapping = vec![
        vec![41, 65, 95, 115],
        vec![19, 30, 44, 53],
        vec![31, 49, 72, 87],
    ];
    let expected = vec![-5, 2, 2, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[-4, 4, -1, 0]));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv = normalize_positive(&super::PrimeLimit::new(7).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn marvel7_from_reduced() {
    let mapping = vec![vec![1, 0, 0, -5], vec![0, 1, 0, 2], vec![0, 0, 1, 2]];
    let expected = vec![-5, 2, 2, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[-4, 4, -1, 0]));
    assert_eq!(Some(expected), only_unison_vector(&mapping));
}

#[test]
fn meantone7() {
    let mapping = vec![vec![31, 49, 72, 87], vec![19, 30, 44, 53]];
    assert!(tempers_out(&mapping, &[-4, 4, -1, 0]));
    assert_eq!(None, only_unison_vector(&mapping));
}

#[test]
fn meantone7_redundant() {
    let mapping = vec![
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
        vec![12, 19, 28, 34],
    ];
    assert!(tempers_out(&mapping, &[-4, 4, -1, 0]));
    assert_eq!(None, only_unison_vector(&mapping));
}

#[test]
fn uv_1575_1573() {
    // This failed with the initial implementation
    let mapping = vec![
        vec![72, 114, 167, 202, 249, 266],
        vec![58, 92, 135, 163, 201, 215],
        vec![87, 138, 202, 244, 301, 322],
        vec![31, 49, 72, 87, 107, 115],
        vec![121, 192, 281, 340, 419, 448],
    ];
    let expected = vec![0, 2, 2, 1, -2, -1];
    assert!(tempers_out(&mapping, &expected));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv = normalize_positive(&super::PrimeLimit::new(13).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn meantone_ets() {
    let limit = super::PrimeLimit::new(5);
    let comma = vec![-4, 4, -1];
    let ets =
        get_ets_tempering_out(&limit.pitches, 3.0, &[comma.clone()], 10);
    assert!(tempers_out(&ets, &comma));
}

#[test]
fn syntonic11_ets() {
    let limit = super::PrimeLimit::new(11);
    let comma = vec![-4, 4, -1, 0, 0];
    let ets =
        get_ets_tempering_out(&limit.pitches, 3.0, &[comma.clone()], 10);
    assert!(tempers_out(&ets, &comma));
}

#[test]
fn marvel11_ets() {
    let limit = super::PrimeLimit::new(11);
    let comma1 = vec![-5, 2, 2, -1, 0];
    let comma2 = vec![2, 3, 1, -2, -1];
    let ets = get_ets_tempering_out(
        &limit.pitches,
        3.0,
        &[comma1.clone(), comma2.clone()],
        10,
    );
    assert!(tempers_out(&ets, &comma1));
    assert!(tempers_out(&ets, &comma2));
}

#[test]
fn porcupine11_ets() {
    let limit = super::PrimeLimit::new(11);
    let comma1 = vec![-1, -3, 1, 0, 1];
    let comma2 = vec![6, -2, 0, -1, 0];
    let comma3 = vec![2, -2, 2, 0, -1];
    let ets = get_ets_tempering_out(
        &limit.pitches,
        3.0,
        &[comma1.clone(), comma2.clone(), comma3.clone()],
        5,
    );
    assert!(tempers_out(&ets, &comma1));
    assert!(tempers_out(&ets, &comma2));
    assert!(tempers_out(&ets, &comma3));
}

#[test]
fn ek_for_search_11() {
    let limit = super::PrimeLimit::new(11).pitches;
    let commas = vec![vec![2, -2, 2, 0, -1], vec![-5, 2, 2, -1, 0]];
    let ek = ek_for_search(&limit, &commas) / 12e2;
    assert!(0.0009400 < ek);
    assert!(ek < 0.0009401);
}

#[test]
fn inherent_errors() {
    let limit = super::PrimeLimit::new(11).pitches;
    let comma = vec![2, -2, 2, 0, -1];
    let ek = inherent_error(&limit, &comma) / 12e2;
    assert!(0.0009400 < ek);
    assert!(ek < 0.0009401);

    let comma = vec![-5, 2, 2, -1, 0];
    let ek = inherent_error(&limit, &comma) / 12e2;
    assert!(0.000357 < ek);
    assert!(ek < 0.000358);

    // The limit does matter
    let limit = super::PrimeLimit::new(7).pitches;
    let comma = vec![-5, 2, 2, -1];
    let ek = inherent_error(&limit, &comma) / 12e2;
    assert!(0.000400 < ek);
    assert!(ek < 0.000401);
}

#[test]
fn meantone5_kernel() {
    let mapping = vec![vec![12, 19, 28], vec![19, 30, 44]];
    let expected = vec![vec![4, -4, 1]];
    let kernel = kernel_basis(&mapping);
    assert_eq!(kernel, expected);
    assert_eq!(Some(expected), saturate(&kernel));
    assert_eq!(kernel, saturated_kernel_basis(&mapping));
    let reduced = super::hermite_normal_form(&mapping);
    assert_eq!(reduced, super::hermite_normal_form(&kernel_basis(&kernel)));
}

#[test]
fn meantone5_redundant_kernel() {
    let mapping = vec![vec![12, 19, 28], vec![19, 30, 44], vec![31, 49, 72]];
    assert_eq!(kernel_basis(&mapping), vec![vec![4, -4, 1]]);
    // The redundant mapping can't be saturated
    assert_eq!(saturate(&mapping), None);
    // But the basis is fine
    assert_eq!(saturated_kernel_basis(&mapping), vec![vec![4, -4, 1]]);
}

#[test]
fn meantone7_kernel() {
    let mapping = vec![vec![12, 19, 28, 34], vec![19, 30, 44, 53]];
    // This is implementation-specific
    let expected = vec![vec![1, 2, -3, 1], vec![0, 12, -13, 4]];
    let kernel = kernel_basis(&mapping);
    assert_eq!(kernel, expected);
    assert_eq!(kernel, saturated_kernel_basis(&mapping));
    let reduced = super::hermite_normal_form(&mapping);
    assert_eq!(reduced, super::hermite_normal_form(&kernel_basis(&kernel)));
}

#[test]
fn meantone7_redundant_kernel() {
    let mapping = vec![
        vec![12, 19, 28, 34],
        vec![19, 30, 44, 53],
        vec![31, 49, 72, 87],
    ];
    // This is implementation-specific
    let expected = vec![vec![1, 2, -3, 1], vec![0, 12, -13, 4]];
    assert_eq!(kernel_basis(&mapping), expected);
    assert_eq!(saturated_kernel_basis(&mapping), expected);
}

#[test]
fn magic11_kernel() {
    let mapping = vec![vec![19, 30, 44, 53, 66], vec![22, 35, 51, 62, 76]];
    let kernel = kernel_basis(&mapping);
    let reduced = super::hermite_normal_form(&mapping);
    assert_eq!(
        super::hermite_normal_form(&kernel),
        super::hermite_normal_form(&saturated_kernel_basis(&mapping)),
    );
    assert_eq!(reduced, super::hermite_normal_form(&kernel_basis(&kernel)));
    assert_eq!(
        reduced,
        super::hermite_normal_form(&saturated_kernel_basis(&kernel)),
    );
}

#[test]
fn marvel11_kernel() {
    let mapping = vec![
        vec![19, 30, 44, 53, 66],
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
    ];
    let kernel = kernel_basis(&mapping);
    let reduced = super::hermite_normal_form(&mapping);
    assert_eq!(
        super::hermite_normal_form(&kernel),
        super::hermite_normal_form(&saturated_kernel_basis(&mapping)),
    );
    assert_eq!(reduced, super::hermite_normal_form(&kernel_basis(&kernel)));
    assert_eq!(
        reduced,
        super::hermite_normal_form(&saturated_kernel_basis(&kernel)),
    );
}

#[test]
fn mystery17_uvs() {
    let mapping = vec![
        vec![29, 46, 67, 81, 100, 107, 119],
        vec![58, 92, 135, 163, 201, 215, 237],
    ];
    let plimit = super::PrimeLimit::new(17);
    let uvs = unison_vector_basis(&plimit.pitches, &mapping);
    let ratios: Vec<_> = uvs
        .iter()
        .map(|uv| super::ratio::get_ratio_or_ket_string(&plimit, uv))
        .collect();

    // TLLL reduction doesn't do a brilliant job,
    // but note that the simplest ratio is quite simple
    assert_eq!(
        vec![
            "352:351",
            "2080:2079",
            "5831:5808",
            "53508:53125",
            "18830774421468890345242624:18277964950103308174248225",
        ],
        ratios,
    );
}

#[test]
fn mystery17_kernel() {
    let mapping = vec![
        vec![29, 46, 67, 81, 100, 107, 119],
        vec![58, 92, 135, 163, 201, 215, 237],
    ];
    let kernel = kernel_basis(&mapping);
    let reduced = super::hermite_normal_form(&mapping);
    assert_eq!(
        super::hermite_normal_form(&kernel),
        super::hermite_normal_form(&saturated_kernel_basis(&mapping)),
    );
    assert_eq!(reduced, super::hermite_normal_form(&kernel_basis(&kernel)));
    assert_eq!(
        reduced,
        super::hermite_normal_form(&saturated_kernel_basis(&kernel)),
    );
}

#[test]
fn saturate_vector() {
    let mapping = vec![vec![2, 4, 6, 8]];
    let expected = vec![vec![1, 2, 3, 4]];
    assert_eq!(saturate(&mapping), Some(expected));
}

#[test]
fn saturate_negative() {
    let mapping = vec![vec![-2, -4, -6, -8]];
    let expected = vec![vec![-1, -2, -3, -4]];
    assert_eq!(saturate(&mapping), Some(expected));
}

#[test]
fn saturate_vector10() {
    let mapping = vec![vec![20, 40, 60, 80]];
    let expected = vec![vec![1, 2, 3, 4]];
    assert_eq!(saturate(&mapping), Some(expected));
}

#[test]
fn saturate_matrix() {
    let mapping = vec![vec![2, 4, 6], vec![3, 4, 5]];
    let expected = vec![vec![1, 0, -1], vec![0, 1, 2]];
    assert_eq!(saturate(&mapping), Some(expected));
}

#[test]
fn saturate_empty() {
    assert_eq!(saturate(&vec![]), Some(vec![]));
}

#[test]
fn saturate_zero() {
    assert_eq!(saturate(&vec![vec![0]]), None);
}

#[test]
fn saturate_redundant() {
    let mapping = vec![vec![1, 2, 3, 4], vec![2, 4, 6, 8]];
    assert_eq!(saturate(&mapping), None);
}

#[test]
fn simple_transpose() {
    let original = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
    let expected = vec![vec![1, 5], vec![2, 6], vec![3, 7], vec![4, 8]];
    assert_eq!(transpose(&original), expected);
    assert_eq!(transpose(&expected), original);
}

#[test]
fn vector_transpose() {
    let original = vec![vec![1, 2, 3, 4]];
    let expected = vec![vec![1], vec![2], vec![3], vec![4]];
    assert_eq!(transpose(&original), expected);
    assert_eq!(transpose(&expected), original);
}

#[test]
fn empty_transpose() {
    let empty: Mapping = vec![];
    let empty_empty: Mapping = vec![vec![]];
    assert_eq!(transpose(&empty), empty_empty);
    assert_eq!(transpose(&empty_empty), empty);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn uneven_transpose() {
    transpose(&vec![vec![1, 2, 3], vec![4, 5]]);
}

#[test]
fn matrix_from_mapping() {
    assert_eq!(
        float_matrix_from_mapping(&vec![vec![1, 2, 3], vec![4, 5, 6]]),
        nalgebra::dmatrix![1.0, 2.0, 3.0; 4.0, 5.0, 6.0],
    )
}

#[test]
fn mapping_from_matrix() {
    assert_eq!(
        mapping_from_float_matrix(
            nalgebra::dmatrix![1.0, 2.0, 3.0; 4.0, 5.0, 6.0]
        ),
        vec![vec![1, 2, 3], vec![4, 5, 6]],
    )
}

#[test]
fn matrix_from_vector() {
    assert_eq!(
        float_matrix_from_mapping(&vec![vec![1, 2, 3]]),
        nalgebra::dmatrix![1.0, 2.0, 3.0],
    )
}

#[test]
fn vector_from_matrix() {
    assert_eq!(
        mapping_from_float_matrix(nalgebra::dmatrix![1.0, 2.0, 3.0]),
        vec![vec![1, 2, 3]],
    )
}

#[test]
fn matrix_from_empty() {
    assert_eq!(float_matrix_from_mapping(&vec![]), nalgebra::dmatrix![])
}

#[test]
fn mapping_from_empty() {
    let empty: Mapping = vec![];
    assert_eq!(mapping_from_float_matrix(nalgebra::dmatrix![]), empty)
}

#[test]
fn tlll_limit11() {
    // Compared to Python implementation
    let plimit = super::PrimeLimit::new(11).pitches;
    let vectors = vec![vec![1, 2, 3, 4, 5], vec![3, 4, 2, 2, 3]];
    let result = tlll(&plimit, &vectors);
    assert_eq!(result, vec![vec![2, 2, -1, -2, -2], vec![5, 6, 1, 0, 1]]);
}

#[test]
fn lll_limit11() {
    // Compared to Python implementation
    let plimit = super::PrimeLimit::new(11).pitches;
    let vectors = vec![vec![1, 2, 3, 4, 5], vec![3, 4, 2, 2, 3]];
    let reducer = LLLReducer::new(&plimit);
    let result = reducer.reduce(&vectors);
    assert_eq!(result, vec![vec![2, 2, -1, -2, -2], vec![5, 6, 1, 0, 1]]);
}

#[test]
fn gram_schmidt_limit11() {
    // Compared to Python implemetation
    let reducer = LLLReducer::new(&super::PrimeLimit::new(11).pitches);
    let (g, m) = reducer.gram_schmidt_orthogonalization(&vec![
        vec![1, 2, 3, 4, 5],
        vec![3, 4, 2, 2, 3],
    ]);
    assert_eq!(g.len(), 2);
    // First row is unchanged from the input so will be exact
    assert_eq!(g[0], vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_between!(2.385371887339598, g[1][0], 2.385371887339600);
    assert_between!(2.770743774679197, g[1][1], 2.770743774679199);
    assert_between!(0.156115662018796, g[1][2], 0.156115662018798);
    assert_between!(-0.458512450641605, g[1][3], -0.458512450641602);
    assert_between!(-0.073140563302006, g[1][4], -0.073140563302003);
    assert_eq!(m.len(), 2);
    // Exact integers in the Python
    assert_eq!(m[0], vec![1.0, 0.0]);
    assert_between!(0.614628112660400, m[1][0], 0.614628112660402);
    assert_eq!(m[1][1], 1.0);
}

#[test]
fn lll_unewighted_prod() {
    let reducer = LLLReducer::new(&vec![1.0, 1.0, 1.0]);
    let prod = reducer.prod(&vec![1.0, 0.0, 0.0], &vec![1.0, 0.0, 0.0]);
    super::assert_between!(0.999999, prod, 1.000001);
    assert_eq!(
        0.0,
        reducer.prod(&vec![1.0, 0.0, 0.0], &vec![0.0, 1.0, 1.0]),
    );
    let prod = reducer.prod(&vec![2.0, 0.0, 0.0], &vec![1.0, 0.0, 0.0]);
    super::assert_between!(1.999999, prod, 2.000001);
    let prod = reducer.prod(&vec![3.0, 3.0, 3.0], &vec![2.0, 2.0, 2.0]);
    super::assert_between!(17.999999, prod, 18.000001);
}

#[test]
fn lll_weighted_prod() {
    let reducer = LLLReducer::new(&vec![2.0, 3.0, 4.0]);
    let prod = reducer.prod(&vec![1.0, 0.0, 0.0], &vec![1.0, 0.0, 0.0]);
    super::assert_between!(3.999999, prod, 4.000001);
    assert_eq!(
        0.0,
        reducer.prod(&vec![1.0, 0.0, 0.0], &vec![0.0, 1.0, 1.0]),
    );
    let prod = reducer.prod(&vec![2.0, 0.0, 0.0], &vec![1.0, 0.0, 0.0]);
    super::assert_between!(7.999999, prod, 8.000001);
    let prod = reducer.prod(&vec![3.0, 3.0, 3.0], &vec![2.0, 2.0, 2.0]);
    super::assert_between!(173.999999, prod, 174.000001);
}

#[test]
fn round_lll_checks() {
    assert_eq!(round_lll(0.4), 0);
    assert_eq!(round_lll(0.5), 0);
    assert_eq!(round_lll(0.500001), 1);
    assert_eq!(round_lll(0.6), 1);
    assert_eq!(round_lll(1.5), 1);
    assert_eq!(round_lll(17.499999999), 17);
    assert_eq!(round_lll(17.5), 17);
    assert_eq!(round_lll(17.500001), 18);
    assert_eq!(round_lll(-0.4999999), 0);
    assert_eq!(round_lll(-0.5), -1);
    assert_eq!(round_lll(-0.5000001), -1);
}
