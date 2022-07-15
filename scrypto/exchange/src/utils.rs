use scrypto::prelude::*;

pub fn sort_addresses(
  address0: ResourceAddress,
  address1: ResourceAddress
) -> (ResourceAddress, ResourceAddress) {
  return match address0.to_vec() > address1.to_vec() {
    true => (address0, address1),
    false => (address1, address0)
  }
}

pub fn sort_buckets(
  bucket0: Bucket,
  bucket1: Bucket
) -> (Bucket, Bucket) {
  let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
    bucket0.resource_address(),
    bucket1.resource_address()
  );

  return match bucket0.resource_address() == addresses.0 {
    true => (bucket0, bucket1),
    false => (bucket1, bucket0)
  }
}

pub fn pair_symbol(
  address0: ResourceAddress,
  address1: ResourceAddress
) -> String {
  let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(address0, address1);
  let names: (String, String) = (
    match borrow_resource_manager!(addresses.0).metadata().get("symbol") {
      Some(s) => format!("{}", s),
      None => format!("{}", addresses.0)
    },
    match borrow_resource_manager!(addresses.1).metadata().get("symbol") {
      Some(s) => format!("{}", s),
      None => format!("{}", addresses.1)
    }
  );

  // Format the names and return them.
  return format!("{}-{}", names.0, names.1);
}