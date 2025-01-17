// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use ark_ff::Field;
use itertools::izip;

use super::*;

///
pub fn check_arith_gate<F: Field>(
    setup: &SetupComposer<F>,
    proving: &ProvingComposer<F>,
    pub_inputs: &[F],
) {
    assert_eq!(setup.n, proving.n, "circuit size in setup not equals to proving");
    assert_eq!(setup.pp.size(), pub_inputs.len(), "arity of public inputs in setup is not correct");
    assert_eq!(proving.pi.size(), pub_inputs.len(), "arity of public inputs in proving is not correct");
    for (i, (x, y)) in proving.pi.get_vals().zip(pub_inputs.iter()).enumerate() {
        assert_eq!(x, y, "public input value at {:?} is not correct", i);
    }

    let gates = izip!(
        setup.q_m.iter(),
        setup.q_l.iter(),
        setup.q_r.iter(),
        setup.q_o.iter(),
        setup.q_c.iter(),
        proving.w_l.iter(),
        proving.w_r.iter(),
        proving.w_o.iter(),
        proving.pi.as_evals(proving.n),
    );

    for (
        i,
        (
            &q_m,
            &q_l,
            &q_r,
            &q_o,
            &q_c,
            &w_l,
            &w_r,
            &w_o,
            pi,
        ),
    ) in gates.enumerate() {
        let a = proving.var_map.value_of_var(w_l);
        let b = proving.var_map.value_of_var(w_r);
        let c = proving.var_map.value_of_var(w_o);
        let out = (q_m * a * b) + (q_l * a) + (q_r * b) + (q_o * c) + pi + q_c;
        if !out.is_zero() {
            #[cfg(feature = "trace")]
            {
                let mut backtrace = setup.backtrace[i].clone();
                backtrace.resolve();
                println!("{:?}", backtrace);
            }
            panic!("arithmetic gate at {:?} is not satisfied", i);
        }
    }
}

///
pub fn test_gate_constraints<F, I, P>(process: P, pub_inputs: &[F])
where
    F: Field,
    I: IntoIterator<Item = (LTVariable<F>, F)>,
    P: Fn(&mut ConstraintSystem<F>) -> I,
{
    let mut setup = ConstraintSystem::new(true, Default::default());
    let mut proving = ConstraintSystem::new(false, Default::default());

    process(&mut setup);
    let setup: SetupComposer<F> = setup.composer.into();

    let var_map = process(&mut proving);
    let proving: ProvingComposer<F> = proving.composer.into();
    for (lt_var, expect) in var_map {
        let actual = proving.var_map.value_of_lt_var(&lt_var);
        if actual != expect {
            #[cfg(feature = "trace")]
            {
                let backtrace = proving.var_map.backtrace_of_var(lt_var.var);
                if let Some(mut backtrace) = backtrace {
                    backtrace.resolve();
                    println!("{:?}", backtrace);
                }
            }
            panic!("value of variable {:?} is incorrect", lt_var.var);
        }
    }

    check_arith_gate(
        &setup,
        &proving,
        pub_inputs,
    )
}
