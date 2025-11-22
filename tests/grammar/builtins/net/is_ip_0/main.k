import math
import net

schema IPCheck[ipValue]:
    ip: str = ipValue

    """
    # FIXME: built-in func in schema check
    check:
        net.is_IP(ip)
    """

ipcheck = IPCheck("192.168.0.1") {}

isip00 = net.is_IP("192.168.0,1")
isip01 = net.is_IP("192.168.0.")
isip02 = net.is_IP("192.168.")
isip03 = net.is_IP("192.")
isip04 = net.is_IP("256.168.0,1")
isip05 = net.is_IP("255.255.0.-1")
isip06 = net.is_IP("192.168.")
isip07 = net.is_IP("192.0022.1.1")
isip08 = net.is_IP("492.10.123.12313")
isip09 = net.is_IP("0xFF.0xFF.0xFF.0xFF")

isip10 = net.is_IP("255.255.0.1")
isip11 = net.is_IP("1.0.0.0")
isip12 = net.is_IP("192.190.0.1")
isip14 = net.is_IP("128.0.0.0")
isip15 = net.is_IP("172.16.0.0")
isip16 = net.is_IP("172.31.255.255")
isip17 = net.is_IP("169.254.0.1")
isip18 = net.is_IP("191.255.255.255")
isip19 = net.is_IP("223.255.255.0")
