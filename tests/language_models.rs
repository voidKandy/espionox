use consoxide::language_models::bert;

#[cfg(test)]
#[ignore]
#[test]
fn test_embedding() {
    let embedding = bert::embed(&["nut"]);
    println!("{:?}", embedding);
    assert!(embedding.is_ok());
}
