use consoxide::language_models::huggingface::sentence_embeddings::embed;

#[cfg(test)]
#[ignore]
#[test]
fn test_embedding() {
    let embedding = embed(&[r#"
[ERROR] [2023-07-22 13:45:21] - An unexpected error occurred. Please try again later or contact support for assistance.
"#]);
    let embedding2 = embed(&[r#"running 1 test
test fail ... FAILED

failures:

---- fail stdout ----
thread 'fail' panicked at 'assertion failed: false', tests/mod.rs:9:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    fail

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s"#]);
    // println!("{:?}", embedding);

    let embedding3 = embed(&[
        r#"test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/database.rs (target/debug/deps/database-9ba545a492eb624f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/io.rs (target/debug/deps/io-a07b4f64099c6ef2)

running 0 tests"#,
    ]);
    fn l2_norm(vector1: &Vec<f32>, vector2: &Vec<f32>) -> f32 {
        if vector1.len() != vector2.len() {
            panic!("Vector dimensions must be the same.");
        }

        let sum_of_squares: f32 = vector1
            .iter()
            .zip(vector2.iter())
            .map(|(&x, &y)| (x - y).powi(2))
            .sum();

        sum_of_squares.sqrt()
    }

    fn dot_product(vector1: &Vec<f32>, vector2: &Vec<f32>) -> f32 {
        if vector1.len() != vector2.len() {
            panic!("Vector dimensions must be the same.");
        }

        vector1
            .iter()
            .zip(vector2.iter())
            .map(|(&x, &y)| x * y)
            .sum()
    }

    fn vector_magnitude(vector: &Vec<f32>) -> f32 {
        vector.iter().map(|&x| x * x).sum::<f32>().sqrt()
    }

    fn cosine_angle_between_vectors(vector1: &Vec<f32>, vector2: &Vec<f32>) -> f32 {
        let dot_product = dot_product(vector1, vector2);
        let magnitude1 = vector_magnitude(vector1);
        let magnitude2 = vector_magnitude(vector2);

        dot_product / (magnitude1 * magnitude2)
    }

    // let l2_norm_result = l2_norm(
    //     embedding.unwrap().iter().next().unwrap(),
    //     embedding2.unwrap().iter().next().unwrap(),
    // );
    // println!("L2 Norm: {}", l2_norm_result);
    let cosine_angle = cosine_angle_between_vectors(
        embedding.unwrap().iter().next().unwrap(),
        embedding3.unwrap().iter().next().unwrap(),
    );
    println!("Cosine of angle between vectors: {}", cosine_angle);
    // assert!(embedding.is_ok());
    assert!(false);
}
