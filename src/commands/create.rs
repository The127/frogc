pub fn run(spec: String, container_id: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating container {} with spec {}", container_id, spec);
    Ok(())
}