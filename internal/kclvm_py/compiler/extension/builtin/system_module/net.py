#! /usr/bin/env python3
import ipaddress as _ip
import socket as _socket

import kclvm.kcl.error as kcl_error


def check_empty_str(ip):
    if not isinstance(ip, str) or ip.strip() == "":
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.EvaluationError_TYPE,
            arg_msg="ip must be non-empty string",
        )


def _get_ip(ip):
    try:
        return _ip.ip_address(ip)
    except ValueError:
        return None


def KMANGLED_split_host_port(ip_end_point: str):
    """
    split the 'host' and 'port' from the ip end point
    """
    check_empty_str(ip_end_point)
    return ip_end_point.split(":")


def KMANGLED_join_host_port(host, port):
    """
    merge the 'host' and 'port'
    """
    return "{}:{}".format(host, port)


def KMANGLED_fqdn(name=""):
    """
    get Fully Qualified Domain Name (FQDN)
    """
    return _socket.getfqdn(str(name))


def KMANGLED_parse_IP(ip):
    """
    parse 'ip' to a real IP address
    """
    return _get_ip(ip)


def KMANGLED_to_IP4(ip):
    """
    get the IP4 form of 'ip'
    """
    return str(_get_ip(ip))


def KMANGLED_to_IP16(ip):
    """
    get the IP16 form of 'ip'
    """
    return int(_get_ip(ip))


def KMANGLED_IP_string(ip: str):
    """
    get the IP string
    """
    return _get_ip(ip)


def KMANGLED_is_IPv4(ip: str):
    """
    whether 'ip' is a IPv4 one
    """
    ip = _get_ip(ip)
    return isinstance(ip, _ip.IPv4Address)


def KMANGLED_is_IP(ip: str) -> bool:
    """
    whether ip is a valid ip address

    Parameters
    ----------
    - ip: input ip address

    Returns
    -------
    - is_ip: a bool type return value
    """
    ip = _get_ip(ip)
    return ip is not None


def KMANGLED_is_loopback_IP(ip: str):
    """
    whether 'ip' is a loopback one
    """
    ip = _get_ip(ip)
    return ip.is_loopback if ip else False


def KMANGLED_is_multicast_IP(ip: str):
    """
    whether 'ip' is a multicast one
    """
    ip = _get_ip(ip)
    return ip.is_multicast if ip else False


def KMANGLED_is_interface_local_multicast_IP(ip: str):
    """
    whether 'ip' is a interface, local and multicast one
    """
    try:
        ip = _ip.ip_interface(ip)
        return (ip.is_site_local and ip.is_multicast) if ip else False
    except ValueError:
        return False


def KMANGLED_is_link_local_multicast_IP(ip: str):
    """
    whether 'ip' is a link local and multicast one
    """
    ip = _get_ip(ip)
    return (ip.is_link_local and ip.is_multicast) if ip else False


def KMANGLED_is_link_local_unicast_IP(ip: str):
    """
    whether 'ip' is a link local and unicast one
    """
    ip = _get_ip(ip)
    return (ip.is_link_local and not ip.is_multicast) if ip else False


def KMANGLED_is_global_unicast_IP(ip: str):
    """
    whether 'ip' is a global and unicast one
    """
    ip = _get_ip(ip)
    return (ip.is_global and not ip.is_multicast) if ip else False


def KMANGLED_is_unspecified_IP(ip: str):
    """
    whether 'ip' is a unspecified one
    """
    ip = _get_ip(ip)
    return ip.is_unspecified if ip else False
