use rscl_proc::rscl;

rscl! {
    kernel fn Add (const len: u64, pub lhs: &[f32], pub rhs: &[f32], pub out: &mut [f32]) {
        let i = [1, 2, 3];
        i /= 2;
    }
}