use ofdb_entities::address::Address;

pub trait GeoCodingGateway {
    fn resolve_address_lat_lng(&self, addr: &Address) -> Option<(f64, f64)>;
}
