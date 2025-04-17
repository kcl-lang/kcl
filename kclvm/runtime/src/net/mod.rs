//! Copyright The KCL Authors. All rights reserved.

use crate::*;
use cidr::{IpCidr, Ipv4Cidr, Ipv6Cidr};
use itertools::Itertools;
use std::net::IpAddr;
use std::net::IpAddr::V4;
use std::net::IpAddr::V6;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

// split_host_port(ip_end_point: str) -> List[str]

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_split_host_port(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(ip_end_point) = get_call_arg(args, kwargs, 0, Some("ip_end_point")) {
        let ip_end_point_str = ip_end_point.as_str();
        match ip_end_point_str.rsplit_once(':') {
            None => panic!(
                "ip_end_point \"{}\" missing port",
                ip_end_point_str.escape_default()
            ),
            Some((host, port)) => {
                if host.starts_with('[') {
                    match ip_end_point_str.find(']') {
                        None => panic!(
                            "ip_end_point \"{}\" missing ']'",
                            ip_end_point_str.escape_default()
                        ),
                        Some(end) => {
                            if end > host.len() || !ip_end_point_str[end + 1..].starts_with(':') {
                                panic!(
                                    "ip_end_point \"{}\" missing port",
                                    ip_end_point_str.escape_default()
                                );
                            }
                            if end < host.len() - 1 {
                                panic!(
                                    "ip_end_point \"{}\" too many colons",
                                    ip_end_point_str.escape_default()
                                );
                            }
                            if ip_end_point_str[1..].contains('[') {
                                panic!(
                                    "ip_end_point \"{}\" unexpected '['",
                                    ip_end_point_str.escape_default()
                                );
                            }
                            if port.contains(']') {
                                panic!(
                                    "ip_end_point \"{}\" unexpected ']'",
                                    ip_end_point_str.escape_default()
                                );
                            }
                            return ValueRef::list(Some(&[
                                &ValueRef::str(&host[1..end]),
                                &ValueRef::str(port),
                            ]))
                            .into_raw(ctx);
                        }
                    }
                }
                if host.contains(':') {
                    panic!(
                        "ip_end_point \"{}\" too many colons",
                        ip_end_point_str.escape_default()
                    );
                }
                if ip_end_point_str[1..].contains('[') {
                    panic!(
                        "ip_end_point \"{}\" unexpected '['",
                        ip_end_point_str.escape_default()
                    );
                }
                if ip_end_point_str.contains(']') {
                    panic!(
                        "ip_end_point \"{}\" unexpected ']'",
                        ip_end_point_str.escape_default()
                    );
                }
                return ValueRef::list(Some(&[&ValueRef::str(host), &ValueRef::str(port)]))
                    .into_raw(ctx);
            }
        }
    }

    panic!("split_host_port() missing 1 required positional argument: 'ip_end_point'");
}

// join_host_port(host, port) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_join_host_port(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(host) = get_call_arg(args, kwargs, 0, Some("host")) {
        if let Some(port) = get_call_arg(args, kwargs, 1, Some("port")) {
            if host.as_str().contains(':') {
                return ValueRef::str(format!("[{host}]:{port}").as_ref()).into_raw(ctx);
            }
            return ValueRef::str(format!("{host}:{port}").as_ref()).into_raw(ctx);
        }
    }
    panic!("join_host_port() missing 2 required positional arguments: 'host' and 'port'");
}

// fqdn(name: str = '') -> str

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_fqdn(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    use std::net::ToSocketAddrs;
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let name = get_call_arg_str(args, kwargs, 0, Some("name")).unwrap_or_default();
    let hostname = if name.is_empty() {
        match hostname::get() {
            Ok(name) => name.to_string_lossy().into_owned(),
            Err(_) => return ValueRef::str("").into_raw(ctx),
        }
    } else {
        name
    };
    match (hostname.as_str(), 0).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                match dns_lookup::lookup_addr(&addr.ip()) {
                    Ok(fqdn) => ValueRef::str(&fqdn),
                    Err(_) => ValueRef::str(&hostname),
                }
            } else {
                ValueRef::str(&hostname)
            }
        }
        Err(_) => ValueRef::str(&hostname),
    }
    .into_raw(ctx)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_fqdn(
    _ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    panic!("fqdn() do not support the WASM target");
}

// parse_IP(ip) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_parse_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    kclvm_net_IP_string(ctx, args, kwargs)
}

// to_IP4(ip) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_to_IP4(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    kclvm_net_IP_string(ctx, args, kwargs)
}

// to_IP16(ip) -> int

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_to_IP16(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    kclvm_net_IP_string(ctx, args, kwargs)
}

// IP_string(ip: str) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_IP_string(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            let s = format!("{addr}");
            return ValueRef::str(s.as_ref()).into_raw(ctx);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            let s = format!("{addr}");
            return ValueRef::str(s.as_ref()).into_raw(ctx);
        }

        return ValueRef::str("").into_raw(ctx);
    }

    panic!("IP_string() missing 1 required positional argument: 'ip'");
}

// is_IPv4(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_IPv4(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(_addr) = Ipv4Addr::from_str(ip.as_ref()) {
            return kclvm_value_True(ctx);
        }
        if let Ok(_addr) = Ipv6Addr::from_str(ip.as_ref()) {
            return kclvm_value_False(ctx);
        }

        return kclvm_value_False(ctx);
    }

    panic!("is_IPv4() missing 1 required positional argument: 'ip'");
}

// is_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if Ipv4Addr::from_str(ip.as_ref()).is_ok() || Ipv6Addr::from_str(ip.as_ref()).is_ok() {
            kclvm_value_True(ctx)
        } else {
            kclvm_value_False(ctx)
        }
    } else {
        panic!("is_IP() missing 1 required positional argument: 'ip'");
    }
}

// is_loopback_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_loopback_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            let x = addr.is_loopback();
            return kclvm_value_Bool(ctx, x as i8);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            let x = addr.is_loopback();
            return kclvm_value_Bool(ctx, x as i8);
        }

        return kclvm_value_False(ctx);
    }

    panic!("is_loopback_IP() missing 1 required positional argument: 'ip'");
}

// is_multicast_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_multicast_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            let x = addr.is_multicast();
            return kclvm_value_Bool(ctx, x as i8);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            let x = addr.is_multicast();
            return kclvm_value_Bool(ctx, x as i8);
        }

        return kclvm_value_False(ctx);
    }

    panic!("kclvm_net_is_multicast_IP() missing 1 required positional argument: 'ip'");
}

// is_interface_local_multicast_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_interface_local_multicast_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            // For IPv4, interface-local multicast addresses are in the range 224.0.0.0/24
            let is_interface_local =
                addr.octets()[0] == 224 && addr.octets()[1] == 0 && addr.octets()[2] == 0;
            let x = is_interface_local && addr.is_multicast();
            return kclvm_value_Bool(ctx, x as i8);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            // For IPv6, interface-local multicast addresses start with ff01::/16
            let is_interface_local = addr.segments()[0] == 0xff01;
            let x = is_interface_local && addr.is_multicast();
            return kclvm_value_Bool(ctx, x as i8);
        }
        return kclvm_value_Bool(ctx, 0); // False for invalid IP addresses
    }
    panic!("is_interface_local_multicast_IP() missing 1 required positional argument: 'ip'");
}

// is_link_local_multicast_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_link_local_multicast_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            // For IPv4, link-local multicast addresses are in the range 224.0.0.0/24
            let is_link_local_multicast =
                addr.octets()[0] == 224 && addr.octets()[1] == 0 && addr.octets()[2] == 0;
            let x = is_link_local_multicast && addr.is_multicast();
            return kclvm_value_Bool(ctx, x as i8);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            // For IPv6, link-local multicast addresses start with ff02::/16
            let is_link_local_multicast = addr.segments()[0] == 0xff02;
            let x = is_link_local_multicast && addr.is_multicast();
            return kclvm_value_Bool(ctx, x as i8);
        }
        return kclvm_value_Bool(ctx, 0); // False for invalid IP addresses
    }

    panic!("is_link_local_multicast_IP() missing 1 required positional argument: 'ip'");
}

// is_link_local_unicast_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_link_local_unicast_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            let x = addr.is_link_local() && (!addr.is_multicast());
            return kclvm_value_Bool(ctx, x as i8);
        }
        if let Ok(_addr) = Ipv6Addr::from_str(ip.as_ref()) {
            let x = Ipv6Addr_is_unicast_link_local(&_addr) && (!_addr.is_multicast());
            return kclvm_value_Bool(ctx, x as i8);
        }
        return kclvm_value_False(ctx);
    }

    panic!("is_link_local_unicast_IP() missing 1 required positional argument: 'ip'");
}

#[allow(non_camel_case_types, non_snake_case)]
pub const fn Ipv6Addr_is_unicast_link_local(_self: &Ipv6Addr) -> bool {
    (_self.segments()[0] & 0xffc0) == 0xfe80
}

// is_global_unicast_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_global_unicast_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            let x = Ipv4Addr_is_global(&addr) && (!addr.is_multicast());
            return kclvm_value_Bool(ctx, x as i8);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            return kclvm_value_Bool(ctx, addr.is_multicast() as i8);
        }

        return kclvm_value_False(ctx);
    }

    panic!("is_global_unicast_IP() missing 1 required positional argument: 'ip'");
}

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_parse_CIDR(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(cidr) = get_call_arg(args, kwargs, 0, Some("cidr")) {
        if let Ok(cidr) = IpCidr::from_str(&cidr.as_str()) {
            let ip = ValueRef::str(&cidr.first_address().to_string());
            let mask = ValueRef::int(cidr.network_length().into());
            return ValueRef::dict(Some(&[("ip", &ip), ("mask", &mask)])).into_raw(ctx);
        }
        return ValueRef::dict(None).into_raw(ctx);
    }

    panic!("parse_CIDR() missing 1 required positional argument: 'cidr'");
}

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_hosts_in_CIDR(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(cidr) = get_call_arg_str(args, kwargs, 0, Some("cidr")) {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            let ip = parts[0];
            let mask = parts[1];
            if let Ok(ip) = Ipv4Addr::from_str(ip) {
                if let Ok(mask) = mask.parse::<u8>() {
                    let mask = u32::from_be_bytes(ip.octets()) & !((1 << (32 - mask)) - 1);
                    let mut hosts = vec![];
                    for i in 1..(1 << (32 - mask)) - 1 {
                        let ip = u32::from_be_bytes(ip.octets()) + i;
                        hosts.push(ValueRef::str(Ipv4Addr::from(ip).to_string().as_str()));
                    }
                    let hosts_refs: Vec<&ValueRef> = hosts.iter().collect();
                    return ValueRef::list(Some(&hosts_refs[..])).into_raw(ctx);
                }
            }
        }
        return ValueRef::list(None).into_raw(ctx);
    }

    panic!("hosts_in_CIDR() missing 1 required positional argument: 'cidr'");
}

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_subnets_from_CIDR(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(cidr) = get_call_arg_str(args, kwargs, 0, Some("cidr")) {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            let ip = parts[0];
            let mask = parts[1];
            if let Ok(ip) = Ipv4Addr::from_str(ip) {
                if let Ok(mask) = mask.parse::<u8>() {
                    let mask = u32::from_be_bytes(ip.octets()) & !((1 << (32 - mask)) - 1);
                    let mut subnets = vec![];
                    for i in 1..(1 << (32 - mask)) - 1 {
                        let ip = u32::from_be_bytes(ip.octets()) + i;
                        subnets.push(ValueRef::str(
                            format!("{}/{}", Ipv4Addr::from(ip), mask).as_str(),
                        ));
                    }
                    let subnets_refs: Vec<&ValueRef> = subnets.iter().collect();
                    return ValueRef::list(Some(&subnets_refs)).into_raw(ctx);
                }
            }
        }
        return ValueRef::list(None).into_raw(ctx);
    }

    panic!("subnets_from_CIDR() missing 1 required positional argument: 'cidr'");
}

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_IP_in_CIDR(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    let ip = match get_call_arg_str(args, kwargs, 0, Some("ip")) {
        None => {
            panic!("is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'");
        }
        Some(ip) => match IpAddr::from_str(ip.as_str()) {
            Err(err) => panic!("is_IP_in_CIDR() invalid ip: {}", err),
            Ok(ip) => ip,
        },
    };
    let cidr = match get_call_arg_str(args, kwargs, 1, Some("cidr")) {
        None => {
            panic!("is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'");
        }
        Some(cidr) => match IpCidr::from_str(&cidr.as_str()) {
            Err(err) => panic!("is_IP_in_CIDR() invalid cidr: {}", err),
            Ok(cidr) => cidr,
        },
    };

    if cidr.is_ipv6() {
        match ip {
            IpAddr::V4(ip) => {
                return kclvm_value_Bool(
                    ctx,
                    cidr.contains(&IpAddr::V6(ip.to_ipv6_mapped())) as i8,
                );
            }
            IpAddr::V6(_ip) => {}
        }
    }
    return kclvm_value_Bool(ctx, cidr.contains(&ip) as i8);
}

#[allow(non_camel_case_types, non_snake_case)]
fn Ipv4Addr_is_global(_self: &std::net::Ipv4Addr) -> bool {
    // check if this address is 192.0.0.9 or 192.0.0.10. These addresses are the only two
    // globally routable addresses in the 192.0.0.0/24 range.
    if u32::from_be_bytes(_self.octets()) == 0xc0000009
        || u32::from_be_bytes(_self.octets()) == 0xc000000a
    {
        return true;
    }
    !_self.is_private()
        && !_self.is_loopback()
        && !_self.is_link_local()
        && !_self.is_broadcast()
        && !_self.is_documentation()
        && !Ipv4Addr_is_shared(_self) // _self.is_shared()
        && !Ipv4Addr_is_ietf_protocol_assignment(_self) // _self.is_ietf_protocol_assignment()
        && !Ipv4Addr_is_reserved(_self) // _self.is_reserved()
        && !Ipv4Addr_is_benchmarking(_self) // _self.is_benchmarking()
        // Make sure the address is not in 0.0.0.0/8
        && _self.octets()[0] != 0
}

#[allow(non_camel_case_types, non_snake_case)]
const fn Ipv4Addr_is_shared(_self: &std::net::Ipv4Addr) -> bool {
    _self.octets()[0] == 100 && (_self.octets()[1] & 0b1100_0000 == 0b0100_0000)
}
#[allow(non_camel_case_types, non_snake_case)]
const fn Ipv4Addr_is_ietf_protocol_assignment(_self: &std::net::Ipv4Addr) -> bool {
    _self.octets()[0] == 192 && _self.octets()[1] == 0 && _self.octets()[2] == 0
}
#[allow(non_camel_case_types, non_snake_case)]
const fn Ipv4Addr_is_reserved(_self: &std::net::Ipv4Addr) -> bool {
    _self.octets()[0] & 240 == 240 && !_self.is_broadcast()
}
#[allow(non_camel_case_types, non_snake_case)]
const fn Ipv4Addr_is_benchmarking(_self: &std::net::Ipv4Addr) -> bool {
    _self.octets()[0] == 198 && (_self.octets()[1] & 0xfe) == 18
}

// is_unspecified_IP(ip: str) -> bool

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_is_unspecified_IP(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Ok(addr) = Ipv4Addr::from_str(ip.as_ref()) {
            return kclvm_value_Bool(ctx, addr.is_unspecified() as i8);
        }
        if let Ok(addr) = Ipv6Addr::from_str(ip.as_ref()) {
            return kclvm_value_Bool(ctx, addr.is_unspecified() as i8);
        }
        return kclvm_value_False(ctx);
    }
    panic!("is_unspecified_IP() missing 1 required positional argument: 'ip'");
}

// CIDR_subnet(cidr: str, additional_bits: int, net_num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_CIDR_subnet(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    let cidr = match get_call_arg(args, kwargs, 0, Some("cidr")) {
        None => {
            panic!("CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'");
        }
        Some(cidr) => match IpCidr::from_str(&cidr.as_str()) {
            Err(err) => {
                panic!("CIDR_subnet() invalid cidr: {}", err)
            }
            Ok(cidr) => cidr,
        },
    };

    let additional_bits = match get_call_arg(args, kwargs, 1, Some("additional_bits")) {
        None => {
            panic!("CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'");
        }
        Some(additional_bits) => additional_bits.must_as_strict_int(),
    };
    if additional_bits < 0 {
        panic!("CIDR_subnet() invalid additional_bits: cannot be negative");
    }

    let net_num = match get_call_arg(args, kwargs, 2, Some("net_num")) {
        None => {
            panic!("CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'");
        }
        Some(net_num) => net_num.must_as_strict_int(),
    };
    if net_num < 0 {
        panic!("CIDR_subnet() invalid net_num: cannot be negative");
    }

    match CIDR_allocate(cidr, additional_bits, net_num) {
        Ok(value) => return value.into_raw(ctx),
        Err(message) => panic!("CIDR_subnet() {}", message),
    };
}

#[allow(non_camel_case_types, non_snake_case)]
fn CIDR_allocate(cidr: IpCidr, additional_bits: i64, net_num: i64) -> Result<ValueRef, String> {
    let len = cidr.network_length() as i64 + additional_bits;
    let new_cidr = match cidr.first_address() {
        V4(ipv4) => {
            if len > 32 {
                return Err(format!("invalid additional_bits: would extend network length to {} bits, which is too long for IPv4", len));
            }
            if net_num >= (1 << additional_bits) {
                return Err(format!(
                    "additional_bits of {} does not accommodate a net_num of {}",
                    additional_bits, net_num
                ));
            }
            match Ipv4Cidr::new(
                Ipv4Addr::from_bits(ipv4.to_bits() + (net_num << (32 - len)) as u32),
                len as u8,
            ) {
                Err(_e) => unreachable!(),
                Ok(cidr) => format!("{}/{}", cidr.first_address(), cidr.network_length()),
            }
        }
        V6(ipv6) => {
            if len > 128 {
                return Err(format!("invalid additional_bits: would extend network length to {} bits, which is too long for IPv6", len));
            }
            if additional_bits < 64 && net_num as u64 >= (1u64 << additional_bits) {
                return Err(format!(
                    "additional_bits of {} does not accommodate a net_num of {}",
                    additional_bits, net_num
                ));
            }
            if len == 0 {
                return Ok(ValueRef::str("::/0"));
            }
            match Ipv6Cidr::new(
                Ipv6Addr::from_bits(ipv6.to_bits() + ((net_num as u128) << (128 - len))),
                len as u8,
            ) {
                Err(_e) => unreachable!(),
                Ok(cidr) => format!("{}/{}", cidr.first_address(), cidr.network_length()),
            }
        }
    };
    Ok(ValueRef::str(new_cidr.as_str()))
}

// CIDR_subnets(cidr: str, additional_bits: [int]) -> [str]

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_CIDR_subnets(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    let cidr = match get_call_arg(args, kwargs, 0, Some("cidr")) {
        None => {
            panic!("CIDR_subnets() missing 2 required positional arguments: 'cidr' and 'additional_bits'");
        }
        Some(cidr) => match IpCidr::from_str(&cidr.as_str()) {
            Err(err) => {
                panic!("CIDR_subnets() invalid cidr: {}", err)
            }
            Ok(cidr) => cidr,
        },
    };

    let additional_bits_valueref = match get_call_arg(args, kwargs, 1, Some("additional_bits")) {
        None => {
            panic!("CIDR_subnets() missing 2 required positional arguments: 'cidr' and 'additional_bits'");
        }
        Some(additional_bits) => additional_bits,
    };
    let additional_bits = additional_bits_valueref.as_list_ref();

    let mut net_num: Vec<i64> = Vec::with_capacity(additional_bits.values.len());
    for new_additional in additional_bits.values.iter() {
        let mut num = 0i64;
        let bits = new_additional.must_as_strict_int();

        if bits < 0 {
            panic!("CIDR_subnets() invalid additional_bits: cannot be negative");
        }
        let new_bits = cidr.network_length() as i64 + bits;
        if cidr.is_ipv4() && new_bits > 32 {
            panic!("CIDR_subnets() invalid additional_bits: would extend network length to {} bits, which is too long for IPv4", new_bits);
        }
        if cidr.is_ipv6() {
            if bits > 63 {
                panic!("CIDR_subnets() invalid additional_bits: cannot extend more than 63 bits")
            }
            if new_bits > 128 {
                panic!("CIDR_subnets() invalid additional_bits: would extend network length to {} bits, which is too long for IPv6", new_bits);
            }
        }

        let mut try_again = true;
        while try_again {
            try_again = false;
            for i in 0..net_num.len() {
                let mut allocated_num = net_num[i];
                let mut allocated_bits = additional_bits.values[i].must_as_strict_int();
                if allocated_bits > bits {
                    allocated_num >>= allocated_bits - bits;
                    allocated_bits = bits
                }
                if allocated_bits < bits {
                    allocated_num <<= bits - allocated_bits;
                }
                if allocated_num == num {
                    num += 1 << (bits - allocated_bits);
                    try_again = true;
                }
            }
        }
        net_num.push(num);
    }

    let mut subnets = Vec::with_capacity(net_num.len());
    for (additional, num) in additional_bits.values.iter().zip_eq(net_num.iter()) {
        if *num as u64 >= (1u64 << additional.must_as_strict_int()) {
            match subnets.pop() {
                None => unreachable!(),
                Some(last) => panic!("CIDR_subnets() not enough remaining address space for a subnet with a prefix of {} bits after {}", cidr.network_length() as i64 + additional.must_as_strict_int(), last)
            }
        }
        let subnet = match CIDR_allocate(cidr, additional.must_as_strict_int(), *num) {
            Ok(value) => value,
            Err(message) => panic!("CIDR_subnets {}", message),
        };
        subnets.push(subnet);
    }
    ValueRef::list(Some(subnets.iter().collect_vec().as_slice())).into_raw(ctx)
}

// CIDR_host(cidr: str, host_num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_CIDR_host(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    let cidr = match get_call_arg(args, kwargs, 0, Some("cidr")) {
        None => {
            panic!("CIDR_host() missing 2 required positional arguments: 'cidr' and 'host_num'");
        }
        Some(cidr) => match IpCidr::from_str(&cidr.as_str()) {
            Err(err) => {
                panic!("CIDR_host() invalid cidr: {}", err)
            }
            Ok(cidr) => cidr,
        },
    };

    let host_num = match get_call_arg(args, kwargs, 1, Some("host_num")) {
        None => {
            panic!("CIDR_host() missing 2 required positional arguments: 'cidr' and 'host_num'");
        }
        Some(net_num) => net_num.must_as_strict_int(),
    };

    let host_len = cidr.family().len() - cidr.network_length();
    let abs_host_num = match host_num < 0 {
        true => -(host_num + 1),
        false => host_num,
    } as u64;
    if host_len < 64 && (1u64 << host_len) <= abs_host_num {
        panic!(
            "CIDR_host() prefix of {} does not accommodate a host numbered {}",
            cidr.network_length(),
            host_num
        );
    }

    let addr = match cidr.first_address() {
        V4(ipv4) => {
            let mut bits = ipv4.to_bits() as i64;
            if host_num < 0 {
                bits += 1i64 << host_len
            }
            bits += host_num;
            Ipv4Addr::from_bits(bits as u32).to_string()
        }
        V6(ipv6) => {
            let mut bits = ipv6.to_bits();
            if host_len == 128 {
                bits = host_num as u128;
            } else {
                let host_bits = match host_num < 0 {
                    true => (1u128 << host_len) - (-(host_num as i128)) as u128,
                    false => host_num as u128,
                };
                bits += host_bits;
            }
            Ipv6Addr::from_bits(bits).to_string()
        }
    };
    return ValueRef::str(addr.as_str()).into_raw(ctx);
}

// CIDR_netmask(cidr: str) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C-unwind" fn kclvm_net_CIDR_netmask(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    let cidr = match get_call_arg(args, kwargs, 0, Some("cidr")) {
        None => {
            panic!("CIDR_netmask() missing 1 required positional argument: 'cidr'");
        }
        Some(cidr) => match IpCidr::from_str(&cidr.as_str()) {
            Err(err) => {
                panic!("CIDR_netmask() invalid cidr: {}", err)
            }
            Ok(cidr) => cidr,
        },
    };

    if cidr.is_ipv6() {
        panic!("CIDR_netmask() IPv6 addresses cannot have a netmask")
    }

    let bits = -1i64 << (32 - cidr.network_length());
    return ValueRef::str(Ipv4Addr::from_bits(bits as u32).to_string().as_str()).into_raw(ctx);
}

#[cfg(test)]
mod test_net {
    use super::*;

    #[test]
    fn test_split_host_port() {
        let cases = [
            (
                ValueRef::str("invalid.invalid:21"),
                ValueRef::list(Some(&[
                    &ValueRef::str("invalid.invalid"),
                    &ValueRef::str("21"),
                ])),
            ),
            (
                ValueRef::str("192.0.2.1:14"),
                ValueRef::list(Some(&[&ValueRef::str("192.0.2.1"), &ValueRef::str("14")])),
            ),
            (
                ValueRef::str("[2001:db8::]:80"),
                ValueRef::list(Some(&[&ValueRef::str("2001:db8::"), &ValueRef::str("80")])),
            ),
        ];
        let mut ctx = Context::default();
        for (ip_end_point, expected) in cases.iter() {
            unsafe {
                let actual = &*kclvm_net_split_host_port(
                    &mut ctx,
                    &ValueRef::list(Some(&[&ip_end_point])),
                    &ValueRef::dict(None),
                );
                assert_eq!(expected, actual);
            }
            unsafe {
                let actual = &*kclvm_net_split_host_port(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[("ip_end_point", ip_end_point)])),
                );
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn test_split_host_port_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "split_host_port() missing 1 required positional argument: 'ip_end_point'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_split_host_port(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic("ip_end_point \"test-host\" missing port", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("test-host")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"test-host:7:80\" too many colons", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("test-host:7:80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[2001:db8::]\" missing port", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[2001:db8::]")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[2001:db8::]80\" missing port", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[2001:db8::]80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[2001:db8::]9:80\" missing port", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[2001:db8::]9:80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[2001:db8::]:9:80\" too many colons", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[2001:db8::]:9:80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[2001:db8:::80\" missing ']'", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[2001:db8:::80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"t[est-host:80\" unexpected '['", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("t[est-host:80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"]test-host:80\" unexpected ']'", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("]test-host:80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[[2001:db8::]:80\" unexpected '['", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[[2001:db8::]:80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        assert_panic("ip_end_point \"[2001:db8::]:]80\" unexpected ']'", || {
            let ctx = Context::new();
            let args = &ValueRef::list(Some(&[&ValueRef::str("[2001:db8::]:]80")]));
            kclvm_net_split_host_port(ctx.into_raw(), args, &ValueRef::dict(None));
        });
        std::panic::set_hook(prev_hook);
    }

    #[test]
    fn test_join_host_port() {
        let cases = [
            (
                ValueRef::str("invalid.invalid"),
                ValueRef::int(21),
                ValueRef::str("invalid.invalid:21"),
            ),
            (
                ValueRef::str("invalid.invalid"),
                ValueRef::str("21"),
                ValueRef::str("invalid.invalid:21"),
            ),
            (
                ValueRef::str("192.0.2.1"),
                ValueRef::int(14),
                ValueRef::str("192.0.2.1:14"),
            ),
            (
                ValueRef::str("192.0.2.1"),
                ValueRef::str("14"),
                ValueRef::str("192.0.2.1:14"),
            ),
            (
                ValueRef::str("2001:db8::"),
                ValueRef::int(14),
                ValueRef::str("[2001:db8::]:14"),
            ),
            (
                ValueRef::str("2001:db8::"),
                ValueRef::str("14"),
                ValueRef::str("[2001:db8::]:14"),
            ),
        ];
        let mut ctx = Context::default();
        for (host, port, expected) in cases.iter() {
            unsafe {
                let actual = &*kclvm_net_join_host_port(
                    &mut ctx,
                    &ValueRef::list(Some(&[&host, &port])),
                    &ValueRef::dict(None),
                );
                assert_eq!(expected, actual);
            }
            unsafe {
                let actual = &*kclvm_net_join_host_port(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[("host", host), ("port", port)])),
                );
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn test_join_host_port_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "join_host_port() missing 2 required positional arguments: 'host' and 'port'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_join_host_port(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "join_host_port() missing 2 required positional arguments: 'host' and 'port'",
            || {
                let mut ctx = Context::new();
                let args =
                    ValueRef::list(Some(&[&ValueRef::str("invalid.invalid")])).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_join_host_port(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "join_host_port() missing 2 required positional arguments: 'host' and 'port'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(Some(&[("host", &ValueRef::str("invalid.invalid"))]))
                    .into_raw(&mut ctx);
                kclvm_net_join_host_port(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "join_host_port() missing 2 required positional arguments: 'host' and 'port'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs =
                    ValueRef::dict(Some(&[("port", &ValueRef::str("80"))])).into_raw(&mut ctx);
                kclvm_net_join_host_port(ctx.into_raw(), args, kwargs);
            },
        );
        std::panic::set_hook(prev_hook);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_parse_CIDR() {
        let cases = [
            (
                ValueRef::str("0.0.0.0/0"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("0.0.0.0")),
                    ("mask", &ValueRef::int(0)),
                ])),
            ),
            (
                ValueRef::str("::/0"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("::")),
                    ("mask", &ValueRef::int(0)),
                ])),
            ),
            (
                ValueRef::str("10.0.0.0/8"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("10.0.0.0")),
                    ("mask", &ValueRef::int(8)),
                ])),
            ),
            (
                ValueRef::str("2001:db8::/56"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("2001:db8::")),
                    ("mask", &ValueRef::int(56)),
                ])),
            ),
            (
                ValueRef::str("10.1.2.3/32"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("10.1.2.3")),
                    ("mask", &ValueRef::int(32)),
                ])),
            ),
            (
                ValueRef::str("2001:db8:1:2:3:4:5:7/128"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("2001:db8:1:2:3:4:5:7")),
                    ("mask", &ValueRef::int(128)),
                ])),
            ),
            (
                ValueRef::str("10.1.2.3"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("10.1.2.3")),
                    ("mask", &ValueRef::int(32)),
                ])),
            ),
            (
                ValueRef::str("2001:db8:1:2:3:4:5:7"),
                ValueRef::dict(Some(&[
                    ("ip", &ValueRef::str("2001:db8:1:2:3:4:5:7")),
                    ("mask", &ValueRef::int(128)),
                ])),
            ),
            (ValueRef::str("10.0.0/8"), ValueRef::dict(None)),
            (ValueRef::str("10.0.0.0/33"), ValueRef::dict(None)),
            (
                ValueRef::str("2001:db8:1:2:3:4:5:7/129"),
                ValueRef::dict(None),
            ),
            (ValueRef::str("0.0.0.0/256"), ValueRef::dict(None)),
            (ValueRef::str("::/256"), ValueRef::dict(None)),
            (ValueRef::str("10.0.0.0/8/8"), ValueRef::dict(None)),
            (ValueRef::str("2001:db8::/56/56"), ValueRef::dict(None)),
            (ValueRef::str("0.0.0.0/-1"), ValueRef::dict(None)),
            (ValueRef::str("::/-1"), ValueRef::dict(None)),
            (ValueRef::str("10.128.0.0/8"), ValueRef::dict(None)),
            (ValueRef::str("2001:db8::/16"), ValueRef::dict(None)),
            (ValueRef::str("10.1.2.3/31"), ValueRef::dict(None)),
            (
                ValueRef::str("2001:db8:1:2:3:4:5:7/127"),
                ValueRef::dict(None),
            ),
        ];
        let mut ctx = Context::default();
        for (cidr, expected) in cases.iter() {
            unsafe {
                let actual = &*kclvm_net_parse_CIDR(
                    &mut ctx,
                    &ValueRef::list(Some(&[&cidr])),
                    &ValueRef::dict(None),
                );
                assert_eq!(expected, actual, "{} positional", cidr);
            }
            unsafe {
                let actual = &*kclvm_net_parse_CIDR(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[("cidr", cidr)])),
                );
                assert_eq!(expected, actual, "{} named", cidr);
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_parse_CIDR_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "parse_CIDR() missing 1 required positional argument: 'cidr'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_parse_CIDR(ctx.into_raw(), args, kwargs);
            },
        );
        std::panic::set_hook(prev_hook);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_is_IP_in_CIDR() {
        let cases = [
            (
                "0.0.0.0/0",
                vec!["0.0.0.0", "255.255.255.255"],
                vec!["::", "2001:db8::"],
            ),
            (
                "::/0",
                vec![
                    "::",
                    "2001:db8::",
                    "10.1.2.3",
                    "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
                ],
                vec![],
            ),
            (
                "10.0.0.0/8",
                vec!["10.0.0.0", "10.1.2.3", "10.255.255.255"],
                vec!["9.255.255.255", "11.0.0.0", "a000::"],
            ),
            (
                "2001:db8::/56",
                vec!["2001:db8::", "2001:db8:0000:ff:ffff:ffff:ffff:ffff"],
                vec![
                    "2001:db7:ffff:ffff:ffff:ffff:ffff:ffff",
                    "2001:db8:0:100::",
                    "10.1.2.3",
                ],
            ),
            (
                "10.1.2.3/32",
                vec!["10.1.2.3"],
                vec!["10.1.2.2", "10.1.2.4", "0a01:0203::"],
            ),
            (
                "2001:db8:1:2:3:4:5:7/128",
                vec!["2001:db8:1:2:3:4:5:7"],
                vec!["2001:db8:1:2:3:4:5:6", "2001:db8:1:2:3:4:5:8", "10.1.2.3"],
            ),
            (
                "10.1.2.3",
                vec!["10.1.2.3"],
                vec!["10.1.2.2", "10.1.2.4", "0a01:0203::"],
            ),
            (
                "2001:db8:1:2:3:4:5:7",
                vec!["2001:db8:1:2:3:4:5:7"],
                vec!["2001:db8:1:2:3:4:5:6", "2001:db8:1:2:3:4:5:8", "10.1.2.3"],
            ),
        ];
        let mut ctx = Context::default();
        for (cidr, expect_in, expect_not_in) in cases.iter() {
            for ip in expect_in.iter() {
                unsafe {
                    let actual = &*kclvm_net_is_IP_in_CIDR(
                        &mut ctx,
                        &ValueRef::list(Some(&[&ValueRef::str(ip), &ValueRef::str(cidr)])),
                        &ValueRef::dict(None),
                    );
                    assert_eq!(
                        &ValueRef::bool(true),
                        actual,
                        "{} in {} positional",
                        ip,
                        cidr
                    );
                }
                unsafe {
                    let actual = &*kclvm_net_is_IP_in_CIDR(
                        &mut ctx,
                        &ValueRef::list(None),
                        &ValueRef::dict(Some(&[
                            ("cidr", &ValueRef::str(cidr)),
                            ("ip", &ValueRef::str(ip)),
                        ])),
                    );
                    assert_eq!(&ValueRef::bool(true), actual, "{} in {} named", ip, cidr);
                }
            }
            for ip in expect_not_in.iter() {
                unsafe {
                    let actual = &*kclvm_net_is_IP_in_CIDR(
                        &mut ctx,
                        &ValueRef::list(Some(&[&ValueRef::str(ip), &ValueRef::str(cidr)])),
                        &ValueRef::dict(None),
                    );
                    assert_eq!(
                        &ValueRef::bool(false),
                        actual,
                        "{} not in {} positional",
                        ip,
                        cidr
                    );
                }
                unsafe {
                    let actual = &*kclvm_net_is_IP_in_CIDR(
                        &mut ctx,
                        &ValueRef::list(None),
                        &ValueRef::dict(Some(&[
                            ("cidr", &ValueRef::str(cidr)),
                            ("ip", &ValueRef::str(ip)),
                        ])),
                    );
                    assert_eq!(
                        &ValueRef::bool(false),
                        actual,
                        "{} not in {} named",
                        ip,
                        cidr
                    );
                }
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_is_IP_in_CIDR_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_is_IP_in_CIDR(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[&ValueRef::str("10.1.2.3")])).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_is_IP_in_CIDR(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs =
                    ValueRef::dict(Some(&[("ip", &ValueRef::str("10.1.2.3"))])).into_raw(&mut ctx);
                kclvm_net_is_IP_in_CIDR(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(Some(&[("cidr", &ValueRef::str("10.0.0.0/8"))]))
                    .into_raw(&mut ctx);
                kclvm_net_is_IP_in_CIDR(ctx.into_raw(), args, kwargs);
            },
        );
        let cases = [
            ("10.0.0/8", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("10.0.0.0/33", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: invalid length for network: Network length 33 is too long for Ipv4 (maximum: 32)"),
            ("2001:db8:1:2:3:4:5:7/129", "2001:db8::", "is_IP_in_CIDR() invalid cidr: invalid length for network: Network length 129 is too long for Ipv6 (maximum: 128)"),
            ("0.0.0.0/256", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("::/256", "2001:db8::", "is_IP_in_CIDR() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("10.0.0.0/8/8", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("2001:db8::/56/56", "2001:db8::", "is_IP_in_CIDR() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("0.0.0.0/-1", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("::/-1", "2001:db8::", "is_IP_in_CIDR() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("10.128.0.0/8", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: host part of address was not zero"),
            ("2001:db8::/16", "2001:db8::", "is_IP_in_CIDR() invalid cidr: host part of address was not zero"),
            ("10.1.2.3/31", "10.0.0.1", "is_IP_in_CIDR() invalid cidr: host part of address was not zero"),
            ("2001:db8:1:2:3:4:5:7/127", "2001:db8::", "is_IP_in_CIDR() invalid cidr: host part of address was not zero"),
            ("10.0.0.0/8", "10.0.0", "is_IP_in_CIDR() invalid ip: invalid IP address syntax"),
            ("2001:db8::/56", "2001:db8:::", "is_IP_in_CIDR() invalid ip: invalid IP address syntax"),
        ];
        for (cidr, ip, expect_error) in cases.iter() {
            assert_panic(expect_error, || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[&ValueRef::str(ip), &ValueRef::str(cidr)]))
                    .into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_is_IP_in_CIDR(ctx.into_raw(), args, kwargs);
            });
        }
        std::panic::set_hook(prev_hook);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_subnet() {
        let cases = [
            ("0.0.0.0/0", 0i64, 0i64, "0.0.0.0/0"),
            ("0.0.0.0/0", 1, 1, "128.0.0.0/1"),
            ("0.0.0.0/0", 8, 10, "10.0.0.0/8"),
            ("0.0.0.0/0", 9, 11, "5.128.0.0/9"),
            ("0.0.0.0/0", 32, 4294967295, "255.255.255.255/32"),
            ("10.0.0.0/8", 9, 11, "10.5.128.0/17"),
            ("10.0.0.0/8", 24, 16777215, "10.255.255.255/32"),
            ("255.1.2.254/31", 1, 1, "255.1.2.255/32"),
            ("255.255.255.254/31", 1, 1, "255.255.255.255/32"),
            ("255.255.255.255/32", 0, 0, "255.255.255.255/32"),
            ("::/0", 0, 0, "::/0"),
            ("::/0", 1, 1, "8000::/1"),
            ("::/0", 16, 10, "a::/16"),
            ("::/0", 17, 11, "5:8000::/17"),
            ("::/0", 63, 9223372036854775807, "ffff:ffff:ffff:fffe::/63"),
            ("::/0", 64, 9223372036854775807, "7fff:ffff:ffff:ffff::/64"),
            (
                "ffff:ffff:ffff:ffff:8000::/65",
                63,
                9223372036854775807,
                "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff/128",
            ),
            ("2001:db8::/56", 8, 5, "2001:db8:0:5::/64"),
            (
                "2001:db8:1:2:3:4:5:fffe/127",
                1,
                1,
                "2001:db8:1:2:3:4:5:ffff/128",
            ),
            ("2001:db8:1:2:3:4:5:7/128", 0, 0, "2001:db8:1:2:3:4:5:7/128"),
        ];
        let mut ctx = Context::default();
        for (cidr, additional_bits, net_num, expected) in cases.iter() {
            unsafe {
                let actual = &*kclvm_net_CIDR_subnet(
                    &mut ctx,
                    &ValueRef::list(Some(&[
                        &ValueRef::str(cidr),
                        &ValueRef::int(*additional_bits),
                        &ValueRef::int(*net_num),
                    ])),
                    &ValueRef::dict(None),
                );
                assert_eq!(
                    &ValueRef::str(expected),
                    actual,
                    "{} {} {} positional",
                    cidr,
                    additional_bits,
                    net_num
                );
            }
            unsafe {
                let actual = &*kclvm_net_CIDR_subnet(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[
                        ("cidr", &ValueRef::str(cidr)),
                        ("additional_bits", &ValueRef::int(*additional_bits)),
                        ("net_num", &ValueRef::int(*net_num)),
                    ])),
                );
                assert_eq!(
                    &ValueRef::str(expected),
                    actual,
                    "{} {} {} named",
                    cidr,
                    additional_bits,
                    net_num
                );
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_subnet_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_subnet(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[&ValueRef::str("10.1.2.3"), &ValueRef::int(1)])).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_subnet(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs =
                    ValueRef::dict(Some(&[("cidr", &ValueRef::str("10.1.2.3")),("additional_bits", &ValueRef::int(1))])).into_raw(&mut ctx);
                kclvm_net_CIDR_subnet(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs =
                    ValueRef::dict(Some(&[("cidr", &ValueRef::str("10.1.2.3")),("net_num", &ValueRef::int(1))])).into_raw(&mut ctx);
                kclvm_net_CIDR_subnet(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnet() missing 3 required positional arguments: 'cidr', 'additional_bits', and 'net_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs =
                    ValueRef::dict(Some(&[("additional_bits", &ValueRef::int(1)),("net_num", &ValueRef::int(1))])).into_raw(&mut ctx);
                kclvm_net_CIDR_subnet(ctx.into_raw(), args, kwargs);
            },
        );
        let cases = [
            ("10.0.0/8", 1i64, 0i64, "CIDR_subnet() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("10.0.0.0/33", 1, 0, "CIDR_subnet() invalid cidr: invalid length for network: Network length 33 is too long for Ipv4 (maximum: 32)"),
            ("2001:db8:1:2:3:4:5:7/129", 1, 0, "CIDR_subnet() invalid cidr: invalid length for network: Network length 129 is too long for Ipv6 (maximum: 128)"),
            ("0.0.0.0/256", 1, 0, "CIDR_subnet() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("::/256", 1, 0, "CIDR_subnet() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("10.0.0.0/8/8", 1, 0, "CIDR_subnet() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("2001:db8::/56/56", 1, 0, "CIDR_subnet() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("0.0.0.0/-1", 1, 0, "CIDR_subnet() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("::/-1", 1, 0, "CIDR_subnet() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("10.128.0.0/8", 1, 0, "CIDR_subnet() invalid cidr: host part of address was not zero"),
            ("2001:db8::/16", 1, 0, "CIDR_subnet() invalid cidr: host part of address was not zero"),
            ("10.1.2.3/31", 1, 0, "CIDR_subnet() invalid cidr: host part of address was not zero"),
            ("2001:db8:1:2:3:4:5:7/127", 1, 0, "CIDR_subnet() invalid cidr: host part of address was not zero"),
            ("10.0.0.0/8", -1, 0, "CIDR_subnet() invalid additional_bits: cannot be negative"),
            ("2001:db8::/64", 1, -1, "CIDR_subnet() invalid net_num: cannot be negative"),
            ("10.0.0.0/8", 25, 0, "CIDR_subnet() invalid additional_bits: would extend network length to 33 bits, which is too long for IPv4"),
            ("2001:db8::/65", 64, 0, "CIDR_subnet() invalid additional_bits: would extend network length to 129 bits, which is too long for IPv6"),
            ("10.0.0.0/8", 8, 256, "CIDR_subnet() additional_bits of 8 does not accommodate a net_num of 256"),
            ("2001:db8::/64", 8, 256, "CIDR_subnet() additional_bits of 8 does not accommodate a net_num of 256"),
        ];
        for (cidr, additional_bits, net_num, expect_error) in cases.iter() {
            assert_panic(expect_error, || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[
                    &ValueRef::str(cidr),
                    &ValueRef::int(*additional_bits),
                    &ValueRef::int(*net_num),
                ]))
                .into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_subnet(ctx.into_raw(), args, kwargs);
            });
        }
        std::panic::set_hook(prev_hook);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_subnets() {
        let cases = [
            ("0.0.0.0/0", vec![0i64], vec!["0.0.0.0/0"]),
            ("0.0.0.0/0", vec![], vec![]),
            ("0.0.0.0/0", vec![1, 1], vec!["0.0.0.0/1", "128.0.0.0/1"]),
            (
                "0.0.0.0/0",
                vec![8, 9, 9, 32, 32],
                vec![
                    "0.0.0.0/8",
                    "1.0.0.0/9",
                    "1.128.0.0/9",
                    "2.0.0.0/32",
                    "2.0.0.1/32",
                ],
            ),
            (
                "0.0.0.0/0",
                vec![8, 32, 32, 9],
                vec!["0.0.0.0/8", "1.0.0.0/32", "1.0.0.1/32", "1.128.0.0/9"],
            ),
            (
                "10.0.0.0/8",
                vec![8, 9, 9, 9, 8],
                vec![
                    "10.0.0.0/16",
                    "10.1.0.0/17",
                    "10.1.128.0/17",
                    "10.2.0.0/17",
                    "10.3.0.0/16",
                ],
            ),
            (
                "10.0.0.0/8",
                vec![8, 10, 8, 9, 10, 10],
                vec![
                    "10.0.0.0/16",
                    "10.1.0.0/18",
                    "10.2.0.0/16",
                    "10.1.128.0/17",
                    "10.1.64.0/18",
                    "10.3.0.0/18",
                ],
            ),
            (
                "255.1.2.254/31",
                vec![1, 1],
                vec!["255.1.2.254/32", "255.1.2.255/32"],
            ),
            ("255.255.255.255/32", vec![0], vec!["255.255.255.255/32"]),
            ("::/0", vec![0], vec!["::/0"]),
            ("::/0", vec![], vec![]),
            ("::/0", vec![1, 1], vec!["::/1", "8000::/1"]),
            (
                "::/0",
                vec![8, 9, 9, 63, 63],
                vec!["::/8", "100::/9", "180::/9", "200::/63", "200:0:0:2::/63"],
            ),
            (
                "::/0",
                vec![8, 63, 63, 9],
                vec!["::/8", "100::/63", "100:0:0:2::/63", "180::/9"],
            ),
            (
                "2001:db8::/65",
                vec![8, 63, 63, 63, 8],
                vec![
                    "2001:db8::/73",
                    "2001:db8:0:0:80::/128",
                    "2001:db8::80:0:0:1/128",
                    "2001:db8::80:0:0:2/128",
                    "2001:db8:0:0:100::/73",
                ],
            ),
            (
                "2001:db8::/65",
                vec![8, 10, 8, 9, 10, 10],
                vec![
                    "2001:db8::/73",
                    "2001:db8:0:0:80::/75",
                    "2001:db8:0:0:100::/73",
                    "2001:db8:0:0:c0::/74",
                    "2001:db8:0:0:a0::/75",
                    "2001:db8:0:0:180::/75",
                ],
            ),
        ];
        let mut ctx = Context::default();
        for (cidr, additional_bits, expected) in cases.iter() {
            let additional_bits_valueref = additional_bits
                .iter()
                .map(|x| ValueRef::int(*x))
                .collect::<Vec<_>>();
            let expected_valueref = expected
                .iter()
                .map(|x| ValueRef::str(*x))
                .collect::<Vec<_>>();
            unsafe {
                let actual = &*kclvm_net_CIDR_subnets(
                    &mut ctx,
                    &ValueRef::list(Some(&[
                        &ValueRef::str(cidr),
                        &ValueRef::list(Some(&additional_bits_valueref.iter().collect::<Vec<_>>())),
                    ])),
                    &ValueRef::dict(None),
                );
                assert_eq!(
                    &ValueRef::list(Some(&expected_valueref.iter().collect::<Vec<_>>())),
                    actual,
                    "{} {:?} positional",
                    cidr,
                    additional_bits,
                );
            }
            unsafe {
                let actual = &*kclvm_net_CIDR_subnets(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[
                        ("cidr", &ValueRef::str(cidr)),
                        (
                            "additional_bits",
                            &ValueRef::list(Some(
                                &additional_bits_valueref.iter().collect::<Vec<_>>(),
                            )),
                        ),
                    ])),
                );
                assert_eq!(
                    &ValueRef::list(Some(&expected_valueref.iter().collect::<Vec<_>>())),
                    actual,
                    "{} {:?} named",
                    cidr,
                    additional_bits,
                );
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_subnets_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "CIDR_subnets() missing 2 required positional arguments: 'cidr' and 'additional_bits'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_subnets(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnets() missing 2 required positional arguments: 'cidr' and 'additional_bits'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[&ValueRef::str("10.1.2.3")])).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_subnets(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnets() missing 2 required positional arguments: 'cidr' and 'additional_bits'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(Some(&[("cidr", &ValueRef::str("10.1.2.3"))]))
                    .into_raw(&mut ctx);
                kclvm_net_CIDR_subnets(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_subnets() missing 2 required positional arguments: 'cidr' and 'additional_bits'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(Some(&[("additional_bits", &ValueRef::int(1))]))
                    .into_raw(&mut ctx);
                kclvm_net_CIDR_subnets(ctx.into_raw(), args, kwargs);
            },
        );
        let cases = [
            ("10.0.0/8", vec![1i64], "CIDR_subnets() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("10.0.0.0/33", vec![1], "CIDR_subnets() invalid cidr: invalid length for network: Network length 33 is too long for Ipv4 (maximum: 32)"),
            ("2001:db8:1:2:3:4:5:7/129", vec![1], "CIDR_subnets() invalid cidr: invalid length for network: Network length 129 is too long for Ipv6 (maximum: 128)"),
            ("0.0.0.0/256", vec![1], "CIDR_subnets() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("::/256", vec![1], "CIDR_subnets() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("10.0.0.0/8/8", vec![1], "CIDR_subnets() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("2001:db8::/56/56", vec![1], "CIDR_subnets() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("0.0.0.0/-1", vec![1], "CIDR_subnets() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("::/-1", vec![1], "CIDR_subnets() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("10.128.0.0/8", vec![1], "CIDR_subnets() invalid cidr: host part of address was not zero"),
            ("2001:db8::/16", vec![1], "CIDR_subnets() invalid cidr: host part of address was not zero"),
            ("10.1.2.3/31", vec![1], "CIDR_subnets() invalid cidr: host part of address was not zero"),
            ("2001:db8:1:2:3:4:5:7/127", vec![1], "CIDR_subnets() invalid cidr: host part of address was not zero"),
            ("10.0.0.0/8", vec![3, 2, -1], "CIDR_subnets() invalid additional_bits: cannot be negative"),
            ("10.0.0.0/8", vec![3, 2, 25], "CIDR_subnets() invalid additional_bits: would extend network length to 33 bits, which is too long for IPv4"),
            ("2001:db8::/32", vec![3, 2, 64], "CIDR_subnets() invalid additional_bits: cannot extend more than 63 bits"),
            ("2001:db8::/66", vec![3, 2, 63], "CIDR_subnets() invalid additional_bits: would extend network length to 129 bits, which is too long for IPv6"),
            ("10.0.0.0/8", vec![1, 1, 1], "CIDR_subnets() not enough remaining address space for a subnet with a prefix of 9 bits after 10.128.0.0/9"),
            ("2001:db8::/126", vec![1, 1, 1], "CIDR_subnets() not enough remaining address space for a subnet with a prefix of 127 bits after 2001:db8::2/127"),
        ];
        for (cidr, additional_bits, expect_error) in cases.iter() {
            assert_panic(expect_error, || {
                let additional_bits_valueref = additional_bits
                    .iter()
                    .map(|x| ValueRef::int(*x))
                    .collect::<Vec<_>>();
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[
                    &ValueRef::str(cidr),
                    &ValueRef::list(Some(&additional_bits_valueref.iter().collect::<Vec<_>>())),
                ]))
                .into_raw(&mut ctx);

                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_subnets(ctx.into_raw(), args, kwargs);
            });
        }
        std::panic::set_hook(prev_hook);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_host() {
        let cases = [
            ("0.0.0.0/0", 0i64, "0.0.0.0"),
            ("0.0.0.0/0", 1, "0.0.0.1"),
            ("0.0.0.0/0", -1, "255.255.255.255"),
            ("0.0.0.0/0", 256, "0.0.1.0"),
            ("0.0.0.0/0", -256, "255.255.255.0"),
            ("0.0.0.0/0", 4294967295, "255.255.255.255"),
            ("0.0.0.0/0", -4294967296, "0.0.0.0"),
            ("10.0.0.0/8", 11, "10.0.0.11"),
            ("10.0.0.0/8", 16777215, "10.255.255.255"),
            ("10.0.0.0/8", -1, "10.255.255.255"),
            ("255.1.2.254/31", 1, "255.1.2.255"),
            ("255.255.255.254/31", 1, "255.255.255.255"),
            ("255.255.255.255/32", 0, "255.255.255.255"),
            ("255.255.255.255/32", -1, "255.255.255.255"),
            ("::/0", 0, "::"),
            ("::/0", 1, "::1"),
            ("::/0", 16, "::10"),
            ("::/0", -1, "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"),
            ("::/0", 9223372036854775807, "::7fff:ffff:ffff:ffff"),
            ("::/0", -9223372036854775808, "ffff:ffff:ffff:ffff:8000::"),
            (
                "2001:db8:0:2:8000::/65",
                9223372036854775807,
                "2001:db8:0:2:ffff:ffff:ffff:ffff",
            ),
            (
                "2001:db8:0:2:8000::/65",
                -9223372036854775808,
                "2001:db8:0:2:8000::",
            ),
            (
                "ffff:ffff:ffff:ffff:8000::/65",
                9223372036854775807,
                "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
            ),
            ("2001:db8::/56", 5, "2001:db8::5"),
            ("2001:db8:1:2:3:4:5:fffe/127", 1, "2001:db8:1:2:3:4:5:ffff"),
            ("2001:db8:1:2:3:4:5:fffe/127", -1, "2001:db8:1:2:3:4:5:ffff"),
            ("2001:db8:1:2:3:4:5:7/128", 0, "2001:db8:1:2:3:4:5:7"),
            ("2001:db8:1:2:3:4:5:7/128", -1, "2001:db8:1:2:3:4:5:7"),
        ];
        let mut ctx = Context::default();
        for (cidr, host_num, expected) in cases.iter() {
            unsafe {
                let actual = &*kclvm_net_CIDR_host(
                    &mut ctx,
                    &ValueRef::list(Some(&[&ValueRef::str(cidr), &ValueRef::int(*host_num)])),
                    &ValueRef::dict(None),
                );
                assert_eq!(
                    &ValueRef::str(expected),
                    actual,
                    "{} {} positional",
                    cidr,
                    host_num
                );
            }
            unsafe {
                let actual = &*kclvm_net_CIDR_host(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[
                        ("cidr", &ValueRef::str(cidr)),
                        ("host_num", &ValueRef::int(*host_num)),
                    ])),
                );
                assert_eq!(
                    &ValueRef::str(expected),
                    actual,
                    "{} {} named",
                    cidr,
                    host_num
                );
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_host_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic(
            "CIDR_host() missing 2 required positional arguments: 'cidr' and 'host_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_host(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_host() missing 2 required positional arguments: 'cidr' and 'host_num'",
            || {
                let mut ctx = Context::new();
                let args =
                    ValueRef::list(Some(&[&ValueRef::str("10.1.2.3/32")])).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_host(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_host() missing 2 required positional arguments: 'cidr' and 'host_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(Some(&[("cidr", &ValueRef::str("10.1.2.3/32"))]))
                    .into_raw(&mut ctx);
                kclvm_net_CIDR_host(ctx.into_raw(), args, kwargs);
            },
        );
        assert_panic(
            "CIDR_host() missing 2 required positional arguments: 'cidr' and 'host_num'",
            || {
                let mut ctx = Context::new();
                let args = ValueRef::list(None).into_raw(&mut ctx);
                let kwargs =
                    ValueRef::dict(Some(&[("host_num", &ValueRef::int(1))])).into_raw(&mut ctx);
                kclvm_net_CIDR_host(ctx.into_raw(), args, kwargs);
            },
        );
        let cases = [
            ("10.0.0/8", 0i64, "CIDR_host() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("10.0.0.0/33", 0, "CIDR_host() invalid cidr: invalid length for network: Network length 33 is too long for Ipv4 (maximum: 32)"),
            ("2001:db8:1:2:3:4:5:7/129", 0, "CIDR_host() invalid cidr: invalid length for network: Network length 129 is too long for Ipv6 (maximum: 128)"),
            ("0.0.0.0/256", 0, "CIDR_host() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("::/256", 0, "CIDR_host() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("10.0.0.0/8/8", 0, "CIDR_host() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("2001:db8::/56/56", 0, "CIDR_host() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("0.0.0.0/-1", 0, "CIDR_host() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("::/-1", 0, "CIDR_host() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("10.128.0.0/8", 0, "CIDR_host() invalid cidr: host part of address was not zero"),
            ("2001:db8::/16", 0, "CIDR_host() invalid cidr: host part of address was not zero"),
            ("10.1.2.3/31", 0, "CIDR_host() invalid cidr: host part of address was not zero"),
            ("2001:db8:1:2:3:4:5:7/127", 0, "CIDR_host() invalid cidr: host part of address was not zero"),
            ("10.0.0.0/24", 256, "CIDR_host() prefix of 24 does not accommodate a host numbered 256"),
            ("10.0.0.0/24", -257, "CIDR_host() prefix of 24 does not accommodate a host numbered -257"),
            ("10.0.0.0/32", 1, "CIDR_host() prefix of 32 does not accommodate a host numbered 1"),
            ("10.0.0.0/32", -2, "CIDR_host() prefix of 32 does not accommodate a host numbered -2"),
            ("0.0.0.0/0", 4294967296, "CIDR_host() prefix of 0 does not accommodate a host numbered 4294967296"),
            ("0.0.0.0/0", -4294967297, "CIDR_host() prefix of 0 does not accommodate a host numbered -4294967297"),
            ("2001:db8::/120", 256, "CIDR_host() prefix of 120 does not accommodate a host numbered 256"),
            ("2001:db8::/120", -257, "CIDR_host() prefix of 120 does not accommodate a host numbered -257"),
            ("2001:db8::/66", 9223372036854775807, "CIDR_host() prefix of 66 does not accommodate a host numbered 9223372036854775807"),
            ("2001:db8::/66", -9223372036854775808, "CIDR_host() prefix of 66 does not accommodate a host numbered -9223372036854775808"),
        ];
        for (cidr, host_num, expect_error) in cases.iter() {
            assert_panic(expect_error, || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[&ValueRef::str(cidr), &ValueRef::int(*host_num)]))
                    .into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_host(ctx.into_raw(), args, kwargs);
            });
        }
        std::panic::set_hook(prev_hook);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_netmask() {
        let cases = [
            ("0.0.0.0/0", "0.0.0.0"),
            ("0.0.0.0/1", "128.0.0.0"),
            ("0.0.0.0/24", "255.255.255.0"),
            ("0.0.0.0/31", "255.255.255.254"),
            ("0.0.0.0/32", "255.255.255.255"),
            ("10.0.0.0/8", "255.0.0.0"),
        ];
        let mut ctx = Context::default();
        for (cidr, expected) in cases.iter() {
            unsafe {
                let actual = &*kclvm_net_CIDR_netmask(
                    &mut ctx,
                    &ValueRef::list(Some(&[&ValueRef::str(cidr)])),
                    &ValueRef::dict(None),
                );
                assert_eq!(&ValueRef::str(expected), actual, "{} positional", cidr,);
            }
            unsafe {
                let actual = &*kclvm_net_CIDR_netmask(
                    &mut ctx,
                    &ValueRef::list(None),
                    &ValueRef::dict(Some(&[("cidr", &ValueRef::str(cidr))])),
                );
                assert_eq!(&ValueRef::str(expected), actual, "{} named", cidr,);
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_CIDR_netmask_invalid() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        let cases = [
            ("10.0.0/8", "CIDR_netmask() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("10.0.0.0/33", "CIDR_netmask() invalid cidr: invalid length for network: Network length 33 is too long for Ipv4 (maximum: 32)"),
            ("0.0.0.0/256", "CIDR_netmask() invalid cidr: couldn't parse length in network: number too large to fit in target type"),
            ("10.0.0.0/8/8", "CIDR_netmask() invalid cidr: couldn't parse address in network: invalid IP address syntax"),
            ("0.0.0.0/-1", "CIDR_netmask() invalid cidr: couldn't parse length in network: invalid digit found in string"),
            ("10.128.0.0/8", "CIDR_netmask() invalid cidr: host part of address was not zero"),
            ("10.1.2.3/31", "CIDR_netmask() invalid cidr: host part of address was not zero"),
            ("2001:db8::/64", "CIDR_netmask() IPv6 addresses cannot have a netmask"),
        ];
        for (cidr, expect_error) in cases.iter() {
            assert_panic(expect_error, || {
                let mut ctx = Context::new();
                let args = ValueRef::list(Some(&[&ValueRef::str(cidr)])).into_raw(&mut ctx);
                let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
                kclvm_net_CIDR_netmask(ctx.into_raw(), args, kwargs);
            });
        }
        std::panic::set_hook(prev_hook);
    }
}
