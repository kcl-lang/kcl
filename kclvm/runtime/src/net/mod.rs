//! Copyright The KCL Authors. All rights reserved.

use cidr::IpCidr;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

use crate::*;

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
        if ip_end_point.is_none() {
            return ValueRef::none().into_raw(ctx);
        }
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
            if host.is_none() || port.is_none() {
                return ValueRef::none().into_raw(ctx);
            }
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
        if cidr.is_none() {
            return ValueRef::none().into_raw(ctx);
        }
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

#[cfg(test)]
mod test_net {
    use super::*;

    #[test]
    fn test_split_host_port() {
        let cases = [
            (ValueRef::none(), ValueRef::none()),
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
            (ValueRef::none(), ValueRef::none(), ValueRef::none()),
            (ValueRef::none(), ValueRef::int(21), ValueRef::none()),
            (ValueRef::none(), ValueRef::str("21"), ValueRef::none()),
            (
                ValueRef::str("invalid.invalid"),
                ValueRef::none(),
                ValueRef::none(),
            ),
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
            (ValueRef::none(), ValueRef::none()),
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
}
