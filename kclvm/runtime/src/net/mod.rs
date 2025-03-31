//! Copyright The KCL Authors. All rights reserved.

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
        match ip_end_point.as_str().rsplit_once(':') {
            None => panic!(
                "ip_end_point \"{}\" missing port",
                ip_end_point.as_str().escape_default()
            ),
            Some((host, port)) => {
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

    if let Some(host) = get_call_arg_str(args, kwargs, 0, Some("host")) {
        if let Some(port) = args.arg_i_int(1, None).or(kwargs.kwarg_int("port", None)) {
            let s = format!("{host}:{port}");
            return ValueRef::str(s.as_ref()).into_raw(ctx);
        }
        if let Some(port) = args.arg_i_str(1, None).or(kwargs.kwarg_str("port", None)) {
            let s = format!("{host}:{port}");
            return ValueRef::str(s.as_ref()).into_raw(ctx);
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

    if let Some(cidr) = get_call_arg_str(args, kwargs, 0, Some("cidr")) {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            let ip = parts[0];
            let mask = parts[1];
            if let Ok(ip) = Ipv4Addr::from_str(ip) {
                if let Ok(mask) = mask.parse::<u8>() {
                    let ip_value = ValueRef::str(ip.to_string().as_str());
                    let mask_value = ValueRef::int(mask as i64);
                    return ValueRef::dict(Some(&[("ip", &ip_value), ("mask", &mask_value)]))
                        .into_raw(ctx);
                }
            }
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

    if let Some(ip) = get_call_arg_str(args, kwargs, 0, Some("ip")) {
        if let Some(cidr) = get_call_arg_str(args, kwargs, 1, Some("cidr")) {
            let parts: Vec<&str> = cidr.split('/').collect();
            if parts.len() == 2 {
                let cidr_ip = parts[0];
                let mask_bits = parts[1];
                let ip_addr = match Ipv4Addr::from_str(&ip) {
                    Ok(ip_addr) => ip_addr,
                    Err(_) => return kclvm_value_False(ctx),
                };
                let cidr_ip_addr = match Ipv4Addr::from_str(cidr_ip) {
                    Ok(cidr_ip_addr) => cidr_ip_addr,
                    Err(_) => return kclvm_value_False(ctx),
                };
                let mask_bits = match mask_bits.parse::<u8>() {
                    Ok(mask_bits) if mask_bits <= 32 => mask_bits,
                    _ => return kclvm_value_False(ctx),
                };
                let mask = !((1 << (32 - mask_bits)) - 1);
                let ip_u32 = u32::from_be_bytes(ip_addr.octets());
                let cidr_ip_u32 = u32::from_be_bytes(cidr_ip_addr.octets());
                let is_in_cidr = (ip_u32 & mask) == (cidr_ip_u32 & mask);
                return kclvm_value_Bool(ctx, is_in_cidr as i8);
            }
        }
        return kclvm_value_False(ctx);
    }

    panic!("is_IP_in_CIDR() missing 2 required positional arguments: 'ip' and 'cidr'");
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
        std::panic::set_hook(prev_hook);
    }
}
