//! What Windows says about its network adapters, for the network watcher and for telling networks
//! apart: one `GetAdaptersAddresses` read turned into plain values, and the gateway's hardware
//! address from the neighbour table.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, ResolveIpNetEntry2, GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST,
    IP_ADAPTER_ADDRESSES_LH, MIB_IPNET_ROW2,
};
use windows_sys::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6};

const ERROR_BUFFER_OVERFLOW: u32 = 111;
/// Loopback (24), virtual adapters such as Wintun, which the VPN's own tunnel uses (53), and tunnels
/// (131) say nothing about how the PC reaches the internet.
const VIRTUAL_TYPES: [u32; 3] = [24, 53, 131];

pub struct Adapter {
    pub index: u32,
    pub name: String,
    pub addresses: Vec<IpAddr>,
    pub gateway: Option<Ipv4Addr>,
    pub metric: u32,
}

/// Adapters that are up and physical (or at least not loopback or a tunnel).
pub fn adapters() -> Vec<Adapter> {
    let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER | GAA_FLAG_INCLUDE_GATEWAYS;
    let mut size: u32 = 16 * 1024;
    let mut buffer: Vec<u64> = Vec::new();
    let mut read = false;
    // The list can grow between the size query and the read, so the call is retried with the size it asks for.
    for _ in 0..3 {
        buffer = vec![0u64; (size as usize).div_ceil(8)];
        // SAFETY: the buffer is `size` bytes, 8-byte aligned, and the call writes only within it.
        let result = unsafe { GetAdaptersAddresses(AF_UNSPEC as u32, flags, std::ptr::null(), buffer.as_mut_ptr().cast(), &mut size) };
        if result == 0 {
            read = true;
            break;
        }
        if result != ERROR_BUFFER_OVERFLOW {
            break;
        }
    }
    if !read {
        return Vec::new();
    }
    let mut list = Vec::new();
    // SAFETY: on success the buffer holds a linked list of adapters that lives as long as `buffer`;
    // every pointer followed below comes from that list and is checked for null first.
    unsafe {
        let mut adapter = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
        while !adapter.is_null() {
            let item = &*adapter;
            adapter = item.Next;
            if item.OperStatus != IfOperStatusUp || VIRTUAL_TYPES.contains(&item.IfType) {
                continue;
            }
            let mut addresses = Vec::new();
            let mut unicast = item.FirstUnicastAddress;
            while !unicast.is_null() {
                if let Some(ip) = ip_of((*unicast).Address.lpSockaddr) {
                    addresses.push(ip);
                }
                unicast = (*unicast).Next;
            }
            let mut gateway = None;
            let mut cursor = item.FirstGatewayAddress;
            while !cursor.is_null() && gateway.is_none() {
                if let Some(IpAddr::V4(v4)) = ip_of((*cursor).Address.lpSockaddr) {
                    gateway = Some(v4);
                }
                cursor = (*cursor).Next;
            }
            list.push(Adapter {
                index: item.Anonymous1.Anonymous.IfIndex,
                name: wide_to_string(item.FriendlyName),
                addresses,
                gateway,
                metric: item.Ipv4Metric,
            });
        }
    }
    list
}

/// # Safety
/// `address` is null or points at a socket address of the size its family says.
unsafe fn ip_of(address: *const SOCKADDR) -> Option<IpAddr> {
    if address.is_null() {
        return None;
    }
    // SAFETY: the caller promises the pointer matches its family.
    unsafe {
        match (*address).sa_family {
            AF_INET => Some(IpAddr::V4(Ipv4Addr::from(u32::from_be((*(address as *const SOCKADDR_IN)).sin_addr.S_un.S_addr)))),
            AF_INET6 => Some(IpAddr::V6(Ipv6Addr::from((*(address as *const SOCKADDR_IN6)).sin6_addr.u.Byte))),
            _ => None,
        }
    }
}

/// # Safety
/// `text` is null or a NUL-terminated UTF-16 string.
unsafe fn wide_to_string(text: *const u16) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: the caller promises a NUL-terminated string; the scan stops at the NUL.
    unsafe {
        let len = (0..).take_while(|&i| *text.add(i) != 0).count();
        String::from_utf16_lossy(std::slice::from_raw_parts(text, len))
    }
}

/// The adapter the PC reaches the internet through: the one with an IPv4 gateway and the lowest metric.
pub fn primary() -> Option<Adapter> {
    adapters().into_iter().filter(|a| a.gateway.is_some()).min_by_key(|a| a.metric)
}

/// The hardware address of `gateway` on adapter `index`, lower case with colons, as on macOS.
pub fn gateway_mac(index: u32, gateway: Ipv4Addr) -> Option<String> {
    // SAFETY: the row is zeroed and filled with an IPv4 address and an interface index before the
    // call, which only writes into it.
    unsafe {
        let mut row: MIB_IPNET_ROW2 = std::mem::zeroed();
        row.InterfaceIndex = index;
        row.Address.si_family = AF_INET;
        row.Address.Ipv4.sin_family = AF_INET;
        row.Address.Ipv4.sin_addr.S_un.S_addr = u32::from(gateway).to_be();
        if ResolveIpNetEntry2(&mut row, std::ptr::null()) != 0 {
            return None;
        }
        let length = (row.PhysicalAddressLength as usize).min(row.PhysicalAddress.len());
        let bytes = &row.PhysicalAddress[..length];
        (length == 6 && bytes.iter().any(|b| *b != 0)).then(|| bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(":"))
    }
}
