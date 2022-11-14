// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::ops::{Add, Mul};
use ark_ff::{FftField, Field};
use ark_poly::{
    EvaluationDomain,
    GeneralEvaluationDomain,
    univariate::DensePolynomial,
    UVPolynomial,
};

/// Returns an iterator over increasing powers of the given `scalar` starting
/// at `0`.
#[inline]
pub fn powers_of<F>(scalar: F) -> impl Iterator<Item = F>
where
    F: Field,
{
    core::iter::successors(Some(F::one()), move |p| Some(*p * scalar))
}

/// Evaluation Domain Extension Trait
pub trait EvaluationDomainExt<F>: EvaluationDomain<F>
where
    F: FftField,
{
    /// Returns the value of `log_2(self.size)`.
    fn log_size_of_group(&self) -> u32;

    /// Returns the inverse of the size in the field.
    fn size_inv(&self) -> F;

    /// Returns a fixed generator of the subgroup.
    fn group_gen(&self) -> F;

    /// Returns the inverse of the fixed generator of the subgroup.
    fn group_gen_inv(&self) -> F;

    /// Returns a fixed multiplicative generator of the finite field.
    fn generator_inv(&self) -> F;
}

impl<F> EvaluationDomainExt<F> for GeneralEvaluationDomain<F>
where
    F: FftField,
{
    #[inline]
    fn log_size_of_group(&self) -> u32 {
        match self {
            GeneralEvaluationDomain::Radix2(domain) => domain.log_size_of_group,
            GeneralEvaluationDomain::MixedRadix(domain) => {
                domain.log_size_of_group
            }
        }
    }

    #[inline]
    fn size_inv(&self) -> F {
        match self {
            GeneralEvaluationDomain::Radix2(domain) => domain.size_inv,
            GeneralEvaluationDomain::MixedRadix(domain) => domain.size_inv,
        }
    }

    #[inline]
    fn group_gen(&self) -> F {
        match self {
            GeneralEvaluationDomain::Radix2(domain) => domain.group_gen,
            GeneralEvaluationDomain::MixedRadix(domain) => domain.group_gen,
        }
    }

    #[inline]
    fn group_gen_inv(&self) -> F {
        match self {
            GeneralEvaluationDomain::Radix2(domain) => domain.group_gen_inv,
            GeneralEvaluationDomain::MixedRadix(domain) => domain.group_gen_inv,
        }
    }

    #[inline]
    fn generator_inv(&self) -> F {
        match self {
            GeneralEvaluationDomain::Radix2(domain) => domain.generator_inv,
            GeneralEvaluationDomain::MixedRadix(domain) => domain.generator_inv,
        }
    }
}

///
pub(crate) fn poly_from_evals<F, D>(
    domain: &D,
    mut evals: Vec<F>,
) -> DensePolynomial<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    domain.ifft_in_place(&mut evals);
    DensePolynomial::from_coefficients_vec(evals)
}

///
pub(crate) fn poly_from_evals_ref<F, D>(
    domain: &D,
    evals: &[F],
) -> DensePolynomial<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    DensePolynomial::from_coefficients_vec(domain.ifft(evals))
}

///
pub(crate) fn poly_from_coset_evals<F, D>(
    domain: &D,
    mut evals: Vec<F>,
) -> DensePolynomial<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    domain.coset_ifft_in_place(&mut evals);
    DensePolynomial::from_coefficients_vec(evals)
}

///
pub(crate) fn evals_from_poly_ref<F, D>(
    domain: &D,
    poly: &DensePolynomial<F>,
) -> Vec<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    domain.fft(poly)
}

///
pub(crate) fn coset_evals_from_poly<F, D>(
    domain: &D,
    mut poly: DensePolynomial<F>,
) -> Vec<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    domain.coset_fft_in_place(&mut poly.coeffs);
    poly.coeffs
}

///
pub(crate) fn coset_evals_from_poly_ref<F, D>(
    domain: &D,
    poly: &DensePolynomial<F>,
) -> Vec<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    domain.coset_fft(poly)
}

/// Linear combination of a series of values
///
/// For values [v_0, v_1,... v_k] returns:
/// v_0 + challenge * v_1 + ... + challenge^k  * v_k
pub(crate) fn lc<T, F>(values: &[T], challenge: F) -> T
where
    T: Mul<F, Output = T> + Add<T, Output = T> + Clone,
    F: Field,
{
    // Ensure valid challenge
    assert_ne!(challenge, F::zero());
    assert_ne!(challenge, F::one());

    let kth_val = match values.last() {
        Some(val) => val.clone(),
        _ => panic!("At least one value must be provided to compute a linear combination")
    };

    values
        .iter()
        .rev()
        .skip(1)
        .fold(kth_val, |acc, val| acc * challenge + val.clone())
}

/// The first lagrange polynomial has the expression:
///
/// ```text
/// L_0(X) = mul_from_1_to_(n-1) [(X - omega^i) / (1 - omega^i)]
/// ```
///
/// with `omega` being the generator of the domain (the `n`th root of unity).
///
/// We use two equalities:
///   1. `mul_from_1_to_(n-1) [1 / (1 - omega^i)] = 1 / n` NOTE: L'Hôpital Principle
///   2. `mul_from_1_to_(n-1) [(X - omega^i)] = (X^n - 1) / (X - 1)`
/// to obtain the expression:
///
/// ```text
/// L_0(X) = (X^n - 1) / n * (X - 1)
/// ```
pub(crate) fn compute_first_lagrange_evaluation<F: Field>(
    n: usize,
    zh_eval: F,
    z: F,
) -> F {
    let n_fr = F::from(n as u64);
    let denom = n_fr * (z - F::one());
    zh_eval * denom.inverse().unwrap()
}

/// Computes lagrange polynomial over `domain` of `index`.
pub(crate) fn compute_lagrange_poly<F, D>(domain: &D, index: usize) -> DensePolynomial<F>
where
    F: FftField,
    D: EvaluationDomain<F>,
{
    let n = domain.size();
    assert!(index < n);

    let mut x_evals = vec![F::zero(); n];
    x_evals[index] = F::one();
    poly_from_evals(domain, x_evals)
}

/// Macro to quickly label polynomials
#[macro_export]
macro_rules! label_polynomial {
    ($poly:expr) => {
        ark_poly_commit::LabeledPolynomial::new(
            stringify!($poly).to_owned(),
            $poly,
            None,
            None,
        )
    };
}

/// Macro to quickly label polynomial commitments
#[macro_export]
macro_rules! label_commitment {
    ($comm:expr) => {
        ark_poly_commit::LabeledCommitment::new(
            stringify!($comm).to_owned(),
            $comm.clone(),
            None,
        )
    };
}

/// Macro to quickly label evaluations
#[macro_export]
macro_rules! label_eval {
    ($eval:expr) => {
        (stringify!($eval).to_owned(), $eval)
    };
}

/// Macro to get appropirate label
#[macro_export]
macro_rules! get_label {
    ($eval:expr) => {
        stringify!($comm).to_owned()
    };
}

///
#[cfg(feature = "parallel")]
#[macro_export]
macro_rules! par_izip {
    // @closure creates a tuple-flattening closure for .map() call. usage:
    // @closure partial_pattern => partial_tuple , rest , of , iterators
    // eg. izip!( @closure ((a, b), c) => (a, b, c) , dd , ee )
    ( @closure $p:pat => $tup:expr ) => {
        |$p| $tup
    };

    // The "b" identifier is a different identifier on each recursion level thanks to hygiene.
    ( @closure $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
        $crate::par_izip!(@closure ($p, b) => ( $($tup)*, b ) $( , $tail )*)
    };

    // unary
    ($first:expr $(,)*) => {
        rayon::iter::IntoParallelIterator::into_par_iter($first)
    };

    // binary
    ($first:expr, $second:expr $(,)*) => {
        $crate::par_izip!($first)
            .zip($second)
    };

    // n-ary where n > 2
    ( $first:expr $( , $rest:expr )* $(,)* ) => {
        $crate::par_izip!($first)
            $(
                .zip($rest)
            )*
            .map(
                $crate::par_izip!(@closure a => (a) $( , $rest )*)
            )
    };
}

// #[cfg(test)]
// mod test {
//     use crate::batch_field_test;

//     use super::*;
//     use ark_bls12_377::Fr as Bls12_377_scalar_field;
//     use ark_bls12_381::Fr as Bls12_381_scalar_field;
//     use ark_ff::Field;
//     use rand_core::OsRng;

//     fn test_correct_lc<F: Field>() {
//         let n_iter = 10;
//         for _ in 0..n_iter {
//             let a = F::rand(&mut OsRng);
//             let b = F::rand(&mut OsRng);
//             let c = F::rand(&mut OsRng);
//             let d = F::rand(&mut OsRng);
//             let e = F::rand(&mut OsRng);
//             let challenge = F::rand(&mut OsRng);
//             let expected = a
//                 + b * challenge
//                 + c * challenge * challenge
//                 + d * challenge * challenge * challenge
//                 + e * challenge * challenge * challenge * challenge;

//             let result = lc(&[a, b, c, d, e], challenge);
//             assert_eq!(result, expected)
//         }
//     }

//     fn test_incorrect_lc<F: Field>() {
//         let n_iter = 10;
//         for _ in 0..n_iter {
//             let a = F::rand(&mut OsRng);
//             let b = F::rand(&mut OsRng);
//             let c = F::rand(&mut OsRng);
//             let d = F::rand(&mut OsRng);
//             let e = F::rand(&mut OsRng);
//             let challenge = F::rand(&mut OsRng);
//             let expected = F::one()
//                 + a
//                 + b * challenge
//                 + c * challenge * challenge
//                 + d * challenge * challenge * challenge
//                 + e * challenge * challenge * challenge * challenge;

//             let result = lc(&[a, b, c, d, e], challenge);
//             assert_eq!(result, expected)
//         }
//     }
//     batch_field_test!(
//         [
//         test_correct_lc
//         ],
//         [
//         test_incorrect_lc
//         ] => Bls12_381_scalar_field
//     );
//     batch_field_test!(
//         [
//         test_correct_lc
//         ],
//         [
//         test_incorrect_lc
//         ] => Bls12_377_scalar_field
//     );
// }
