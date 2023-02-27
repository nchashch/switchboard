pub mod config;
pub mod launcher;

pub fn format_deposit_address(sidechain_number: usize, address: String) -> String {
    let deposit_address: String = format!("s{}_{}_", sidechain_number, address);
    let hash = sha256::digest(deposit_address.as_bytes()).to_string();
    let hash: String = hash[..6].into();
    format!("{}{}", deposit_address, hash)
}
