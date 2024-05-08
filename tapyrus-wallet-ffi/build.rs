fn main() {
    match uniffi::generate_scaffolding("./src/wallet.udl") {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to generate scaffolding: {}", e);
            std::process::exit(1);
        }
    }
}
