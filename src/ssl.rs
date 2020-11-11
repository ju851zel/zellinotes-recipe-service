use rustls::{NoClientAuth, ServerConfig};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;
use std::fs::File;


pub fn init() -> ServerConfig {
// Create configuration
    let mut config = ServerConfig::new(NoClientAuth::new());

// Load key files
    let cert_file = &mut BufReader::new(
        File::open("localhost.crt").unwrap());
    let key_file = &mut BufReader::new(
        File::open("localhost.key").unwrap());

// Parse the certificate and set it in the configuration
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();
    return config
}
