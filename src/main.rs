use std::time::Instant;

use ark_bn254::{Bn254, Fr};
use ark_ff::UniformRand;
use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
use ark_poly_commit::multilinear_pc::MultilinearPC;
use ark_std::test_rng;
use rayon::current_num_threads;

const SUPPORTED_SIZE: usize = 24;

fn main() {
    let thread = current_num_threads();
    

    let mut rng = test_rng();
    let srs = MultilinearPC::<Bn254>::setup(SUPPORTED_SIZE, &mut rng);

    for nv in 8..=SUPPORTED_SIZE {
        println!("start benchmark with {} threads for {} variables", thread, nv);
        let (ck, vk) = MultilinearPC::<Bn254>::trim(&srs, nv);

        let repetition = if nv < 10 {
            20
        } else if nv < 20 {
            10
        } else {
            5
        };
        let mut rng = test_rng();
        let polys = (0..repetition)
            .map(|_| DenseMultilinearExtension::<Fr>::rand(nv, &mut rng))
            .collect::<Vec<_>>();
        let mut rng = test_rng();
        let points = (0..repetition)
            .map(|_| (0..nv).map(|_| Fr::rand(&mut rng)).collect::<Vec<Fr>>())
            .collect::<Vec<_>>();

        // ===========================================================
        let start = Instant::now();
        let commits = polys
            .iter()
            .map(|poly| MultilinearPC::<Bn254>::commit(&ck, poly)).collect::<Vec<_>>();
        println!(
            "Committing MLE with {} variables: {} us",
            nv,
            start.elapsed().as_micros() / repetition as u128
        );

        // ===========================================================
        let start = Instant::now();
        let opens = polys
            .iter()
            .zip(points.iter())
            .map(|(poly, point)| MultilinearPC::<Bn254>::open(&ck, poly, point))
            .collect::<Vec<_>>();

        println!(
            "Opening MLE with {} variables: {} us",
            nv,
            start.elapsed().as_micros() / repetition as u128
        );

        // ===========================================================

        let start = Instant::now();
        let evals = polys
            .iter()
            .zip(points.iter())
            .map(|(poly, point)| poly.evaluate(&point).unwrap())
            .collect::<Vec<_>>();

        println!(
            "Evaluate MLE with {} variables: {} us",
            nv,
            start.elapsed().as_micros() / repetition as u128
        );

        // ===========================================================
        let start = Instant::now();
        commits.iter()
            .zip(points.iter().zip(opens.iter().zip(evals.iter())))
            .for_each(|(com, (point, (open, eval)))| {
               assert!( MultilinearPC::<Bn254>::check(&vk, &com, &point, *eval, &open))
            });

        println!(
            "Verifying MLE with {} variables: {} us\n",
            nv,
            start.elapsed().as_micros() / repetition as u128
        );
    }
}
