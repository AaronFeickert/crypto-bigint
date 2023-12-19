//! Equivalence tests between `crypto_bigint::DynResidue` and `num-bigint`.

use crypto_bigint::{Encoding, Integer, Inverter, NonZero, PrecomputeInverter, U256};
use num_bigint::{BigUint, ModInverse};
use proptest::prelude::*;

type DynResidue = crypto_bigint::modular::DynResidue<{ U256::LIMBS }>;
type DynResidueParams = crypto_bigint::modular::DynResidueParams<{ U256::LIMBS }>;

fn to_biguint(uint: &U256) -> BigUint {
    BigUint::from_bytes_le(uint.to_le_bytes().as_ref())
}

fn retrieve_biguint(residue: &DynResidue) -> BigUint {
    to_biguint(&residue.retrieve())
}

fn reduce(n: &U256, p: DynResidueParams) -> DynResidue {
    let modulus = NonZero::new(p.modulus().clone()).unwrap();
    let n_reduced = n.rem_vartime(&modulus);
    DynResidue::new(&n_reduced, p)
}

prop_compose! {
    fn uint()(bytes in any::<[u8; 32]>()) -> U256 {
        U256::from_le_slice(&bytes)
    }
}
prop_compose! {
    /// Generate a random odd modulus.
    fn modulus()(mut n in uint()) -> DynResidueParams {
        if n.is_even().into() {
            n = n.wrapping_add(&U256::one());
        }

        DynResidueParams::new(&n).expect("modulus should be valid")
    }
}
prop_compose! {
    /// Generate a single residue.
    fn residue()(a in uint(), n in modulus()) -> DynResidue {
        reduce(&a, n.clone())
    }
}

proptest! {
    #[test]
    fn inv(x in uint(), n in modulus()) {
        let x = reduce(&x, n.clone());
        let actual = Option::<DynResidue>::from(x.invert());

        let x_bi = retrieve_biguint(&x);
        let n_bi = to_biguint(n.modulus());
        let expected = x_bi.mod_inverse(&n_bi);

        match (expected, actual) {
            (Some(exp), Some(act)) => {
                let res = x * act;
                prop_assert_eq!(res.retrieve(), U256::ONE);
                prop_assert_eq!(exp, retrieve_biguint(&act).into());
            }
            (None, None) => (),
            (_, _) => panic!("disagreement on if modular inverse exists")
        }
    }

    #[test]
    fn precomputed_inv(x in uint(), n in modulus()) {
        let x = reduce(&x, n.clone());
        let inverter = x.params().precompute_inverter();
        let actual = Option::<DynResidue>::from(inverter.invert(&x));

        let x_bi = retrieve_biguint(&x);
        let n_bi = to_biguint(n.modulus());
        let expected = x_bi.mod_inverse(&n_bi);

        match (expected, actual) {
            (Some(exp), Some(act)) => {
                let res = x * act;
                prop_assert_eq!(res.retrieve(), U256::ONE);
                prop_assert_eq!(exp, retrieve_biguint(&act).into());
            }
            (None, None) => (),
            (_, _) => panic!("disagreement on if modular inverse exists")
        }
    }
}