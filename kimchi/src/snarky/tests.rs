use crate::{
    loc,
    snarky::{
        api::SnarkyCircuit,
        boolean::Boolean,
        errors::{SnarkyError, SnarkyRuntimeError},
    },
    snarky::{checked_runner::RunState, cvar::FieldVar},
};
use ark_ff::One;
use mina_curves::pasta::{Fp, Vesta, VestaParameters};
use mina_poseidon::{
    constants::PlonkSpongeConstantsKimchi,
    sponge::{DefaultFqSponge, DefaultFrSponge},
};

use super::prelude::*;

type BaseSponge = DefaultFqSponge<VestaParameters, PlonkSpongeConstantsKimchi>;
type ScalarSponge = DefaultFrSponge<Fp, PlonkSpongeConstantsKimchi>;

struct TestCircuit {}

struct Priv {
    x: Fp,
    y: Fp,
    z: Fp,
}

impl SnarkyCircuit for TestCircuit {
    type Curve = Vesta;

    type PrivateInput = Priv;
    type PublicInput = Boolean<Fp>;
    type PublicOutput = (Boolean<Fp>, FieldVar<Fp>);

    fn circuit(
        &self,
        sys: &mut RunState<Fp>,
        public: Self::PublicInput,
        private: Option<&Self::PrivateInput>,
    ) -> SnarkyResult<Self::PublicOutput> {
        let x: FieldVar<Fp> = sys.compute(&loc!(), |_| private.unwrap().x)?;
        let y: FieldVar<Fp> = sys.compute(&loc!(), |_| private.unwrap().y)?;
        let z: FieldVar<Fp> = sys.compute(&loc!(), |_| private.unwrap().z)?;

        sys.assert_r1cs(Some("x * y = z"), &loc!(), x, y, z)?;

        let other: Boolean<Fp> = sys.compute(&loc!(), |_| true)?;

        // res1 = public & other
        let res1 = public.and(&other, sys, &loc!());

        // res2 = res1 + 3;
        let three = FieldVar::constant(Fp::from(3));
        let res2 = res1.to_field_var() + three;

        Ok((res1, res2))
    }
}

#[test]
fn test_simple_circuit() {
    // compile
    let test_circuit = TestCircuit {};

    let (mut prover_index, verifier_index) = test_circuit.compile_to_indexes().unwrap();

    // print ASM
    println!("{}", prover_index.asm());

    // prove
    let in1 = Priv {
        x: Fp::one(),
        y: Fp::from(2),
        z: Fp::from(2),
    };

    let public_input = true;
    let debug = true;
    let (proof, public_output) = prover_index
        .prove::<BaseSponge, ScalarSponge>(public_input, in1, debug)
        .unwrap();

    let expected_public_output = (true, Fp::from(4));
    assert_eq!(public_output, expected_public_output);

    // verify proof
    verifier_index.verify::<BaseSponge, ScalarSponge>(proof, public_input, public_output);

    // prove a different execution
    let in2 = Priv {
        x: Fp::one(),
        y: Fp::from(2),
        z: Fp::from(2),
    };
    let public_input = true;
    let debug = true;
    let (proof, public_output) = prover_index
        .prove::<BaseSponge, ScalarSponge>(public_input, in2, debug)
        .unwrap();

    let expected_public_output = (true, Fp::from(4));
    assert_eq!(public_output, expected_public_output);

    // verify proof
    verifier_index.verify::<BaseSponge, ScalarSponge>(proof, public_input, public_output);
}
