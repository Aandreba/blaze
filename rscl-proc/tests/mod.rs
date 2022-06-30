use rscl_proc::rscl;

#[test]
fn a () {
    let add = rscl! {
        kernel fn add (const len: u64, pub lhs: &[f32], pub rhs: &[f32], pub out: &mut [f32]) {
            for i in 0..len {

            }
        }
    };
}