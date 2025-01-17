// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Arithmetic Gates

use ark_ff::{FftField, Field};
use ark_poly::polynomial::univariate::DensePolynomial;
use ark_poly_commit::LabeledPolynomial;
use ark_serialize::*;

use crate::{
    commitment::HomomorphicCommitment,
    proof_system::{ProofEvaluations, WireEvaluations},
};

/// Arithmetic Gates Prover Key
#[derive(Debug, Clone, CanonicalDeserialize, CanonicalSerialize)]
pub struct ProverKey<F: Field> {
    /// Multiplication Selector
    pub q_m: LabeledPolynomial<F, DensePolynomial<F>>,
    /// Left Wire Selector
    pub q_l: LabeledPolynomial<F, DensePolynomial<F>>,
    /// Right Wire Selector
    pub q_r: LabeledPolynomial<F, DensePolynomial<F>>,
    /// Output Wire Selector
    pub q_o: LabeledPolynomial<F, DensePolynomial<F>>,
    /// Constant Selector
    pub q_c: LabeledPolynomial<F, DensePolynomial<F>>,
}

impl<F: Field> ProverKey<F> {
    /// Computes the arithmetic gate contribution to the linearisation
    /// polynomial at the given evaluation points.
    pub(crate) fn compute_linearisation(
        &self,
        wire_evals: &WireEvaluations<F>,
    ) -> DensePolynomial<F> {
        &(self.q_m.polynomial() * (wire_evals.a * wire_evals.b)
            + (self.q_l.polynomial() * wire_evals.a)
            + (self.q_r.polynomial() * wire_evals.b)
            + (self.q_o.polynomial() * wire_evals.c))
            + self.q_c.polynomial()
    }
}

/// Arithmetic Gates Extended Prover Key
#[derive(Debug, Clone, Eq, PartialEq, CanonicalDeserialize, CanonicalSerialize)]
pub struct ExtendedProverKey<F: FftField> {
    /// Multiplication Selector
    pub q_m_coset: Vec<F>,
    /// Left Wire Selector
    pub q_l_coset: Vec<F>,
    /// Right Wire Selector
    pub q_r_coset: Vec<F>,
    /// Output Wire Selector
    pub q_o_coset: Vec<F>,
    /// Constant Selector
    pub q_c_coset: Vec<F>,
}

impl<F: FftField> ExtendedProverKey<F> {
    /// Computes the arithmetic gate contribution to the quotient polynomial at
    /// the element of the domain at the given `index`.
    pub(crate) fn compute_quotient_i(
        &self,
        i: usize,
        a_i: F,
        b_i: F,
        c_i: F,
        pi_i: F,
    ) -> F {
        (a_i * b_i * self.q_m_coset[i])
            + (a_i * self.q_l_coset[i])
            + (b_i * self.q_r_coset[i])
            + (c_i * self.q_o_coset[i])
            + self.q_c_coset[i]
            + pi_i
    }
}

/// Arithmetic Gates Verifier Key
#[derive(CanonicalDeserialize, CanonicalSerialize, derivative::Derivative)]
#[derivative(
    Clone(bound = "PC::Commitment: Clone"),
    Debug(bound = "PC::Commitment: core::fmt::Debug"),
    Eq(bound = "PC::Commitment: Eq"),
    PartialEq(bound = "PC::Commitment: PartialEq")
)]
pub struct VerifierKey<F, PC>
where
    F: Field,
    PC: HomomorphicCommitment<F>,
{
    /// Multiplication Selector Commitment
    pub q_m: PC::Commitment,
    /// Left Selector Commitment
    pub q_l: PC::Commitment,
    /// Right Selector Commitment
    pub q_r: PC::Commitment,
    /// Output Selector Commitment
    pub q_o: PC::Commitment,
    /// Constant Selector Commitment
    pub q_c: PC::Commitment,
}

impl<F, PC> VerifierKey<F, PC>
where
    F: Field,
    PC: HomomorphicCommitment<F>,
{
    /// Computes arithmetic gate contribution to the linearisation polynomial
    /// commitment.
    pub(crate) fn compute_linearisation_commitment(
        &self,
        scalars: &mut Vec<F>,
        points: &mut Vec<PC::Commitment>,
        evaluations: &ProofEvaluations<F>,
    ) {
        scalars.push(evaluations.wire_evals.a * evaluations.wire_evals.b);
        points.push(self.q_m.clone());

        scalars.push(evaluations.wire_evals.a);
        points.push(self.q_l.clone());

        scalars.push(evaluations.wire_evals.b);
        points.push(self.q_r.clone());

        scalars.push(evaluations.wire_evals.c);
        points.push(self.q_o.clone());

        scalars.push(F::one());
        points.push(self.q_c.clone());
    }
}
