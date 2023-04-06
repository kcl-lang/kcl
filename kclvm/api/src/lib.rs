pub mod service;

#[cfg(test)]
pub mod capi_test;

pub mod gpyrpc {
    include!(concat!(env!("OUT_DIR"), "/gpyrpc.rs"));
}
