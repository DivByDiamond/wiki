source: docs/NETWORKING.md
source_sha256: ce13165fef7556796b53863e2fca652314db592d61e9d180f8f64f7902ba32f6
# VXLAN hub: setup and network access

The VXLAN hub joins the in-game connector network to your own machine over UDP, as if the guest were plugged into an Ethernet cable on your PC. From the PC you can reach the guest (ping, `nc`, `ssh` when the guest runs `sshd`), and if you want you can route the guest to the ordinary internet through the PC.

## How it works

```
guest ── connector ── VXLAN hub ══ UDP :4789 ══ PC Linux kernel (vx0) ── PC ── internet
     (in-game network)           (tunnel)        (VXLAN interface)
```

- The tunnel carries Ethernet (L2): the guest's frames arrive unchanged on `vx0` on the PC. Addresses, ARP and routes are yours to configure.
- The server has **one tunnel and one remote peer** (`remoteHost`). Every hub has its own random **VNI** (network number); frames with another VNI are ignored.
- The server accepts datagrams only from the IP address `remoteHost`, from any port, because the Linux kernel chooses the source port itself. Replies go to `remoteHost:remotePort`.
- There is no encryption. The VNI is not a password.

## 1. The Minecraft server

In `config/oc2r-common.toml`, section `[vxlan]`:

```toml
[vxlan]
  enable = true
  remoteHost = "203.0.113.10"   # your PC's address as the server sees it
  remotePort = 4789             # the port the PC listens on
  bindHost = "0.0.0.0"
  bindPort = 4789               # the server's port; the tunnel waits here
```

The config is read at startup, so restart the server. Allow UDP `bindPort` from the PC's address on the server (and in the host's firewall).

Place the VXLAN hub next to network devices (connectors or other network blocks; it does not attach to the device bus). **Right-click the hub with an empty hand** to see its VNI and the ready command for the PC. The same number is logged by the server: `VXLAN hub at … uses VNI …`.

## 2. The PC (Linux, root needed)

```sh
VNI=<number from the hub>
ip link add vx0 type vxlan id $VNI dstport 4789 remote <server-address> local <pc-address>
ip link set vx0 mtu 1450
ip addr add 10.77.0.1/24 dev vx0
ip link set vx0 up
```

`dstport` is the port the PC listens on; it must equal `remotePort` in the server config. `mtu 1450` leaves room for the 50 bytes of VXLAN headers on a normal 1500 network.

The PC's firewall must let UDP 4789 in from the server's address, for example:

```sh
nft add table inet vx
nft "add chain inet vx in { type filter hook input priority 0 ; }"
nft add rule inet vx in ip saddr <server-address> udp dport 4789 accept
```

(this adds to your existing policy; with `ufw` or `firewalld` open the port there). Behind a home router you need a UDP 4789 port forward to the PC; it is easier to join server and PC with a VPN (WireGuard, Tailscale) and use the VPN addresses as `remoteHost` and `local`. A changing PC address does not work: `remoteHost` is fixed.

Windows and macOS have no built-in VXLAN: use Linux in a virtual machine or WSL2 with bridged networking.

## 3. The guest

Give the guest an address on the same network as `vx0`. Guest Linux:

```sh
ip addr add 10.77.0.2/24 dev eth0
ip link set eth0 up
ping 10.77.0.1
```

For OnyxOS: `netcfg list`, then `netcfg set <index> 10.77.0.2 255.255.255.0 0.0.0.0 0.0.0.0` (see below for gateway and DNS when going to the internet). From the PC: `ping 10.77.0.2`. If the guest runs a TCP server the PC can reach it through the tunnel: `ssh user@10.77.0.2` works for a guest with `sshd`. The bundled OnyxOS has no `sshd`.

## 4. Giving the guest the ordinary internet

**Option A, simplest: Internet Card or Internet Gateway.** They reach the internet straight from the server and need neither VXLAN nor any PC setup. Turn on `internetCardEnabled` in the `internet_card` section and put an Internet Card in the computer or an Internet Gateway in the network. Details in [Networking stack](/en/reference/networking).

**Option B: through the PC as a router.** Use it when the guest has to be on your local network or leave from the PC's address. On the PC:

```sh
sysctl -w net.ipv4.ip_forward=1
nft add table ip nat
nft "add chain ip nat post { type nat hook postrouting priority 100 ; }"
nft add rule ip nat post ip saddr 10.77.0.0/24 oifname "<uplink-interface>" masquerade
nft add table inet fw
nft "add chain inet fw fwd { type filter hook forward priority 0 ; }"
nft add rule inet fw fwd iifname "vx0" accept
nft add rule inet fw fwd oifname "vx0" ct state established,related accept
```

In the guest the gateway is the PC's address on `vx0` and the DNS server is any reachable one:

```sh
ip route add default via 10.77.0.1
echo "nameserver 1.1.1.1" > /etc/resolv.conf
ping 1.1.1.1
```

In OnyxOS: `netcfg set <index> 10.77.0.2 255.255.255.0 10.77.0.1 1.1.1.1`. Routing and NAT here are standard Linux configuration; ARP and `ping` between the Linux kernel and the hub were verified through the tunnel, the guest's internet access by this route was not tested separately.

Do not combine an Internet Card and a default route through the PC unless you know which path the guest takes: the guest needs exactly one default gateway.

## Checking and troubleshooting

On the PC: `ip -s link show vx0`, `ip neigh show dev vx0`, `tcpdump -ni any udp port 4789`. On the server read the log: `VXLAN hub at … uses VNI …`.

| Symptom | Cause |
|---|---|
| Guest `ping`: `Destination Host Unreachable`, frames leave in `tcpdump`, no answers | VNI on the PC differs from the hub; `enable = false`; server not restarted; guest address not in the `vx0` network |
| `tcpdump` on the PC sees no replies from the server | wrong `remoteHost`/`remotePort`; PC or router firewall blocks UDP 4789 |
| Frames do not reach the server | server firewall blocks UDP `bindPort`; `remoteHost` is not the address the packets really come from (NAT) |
| Ping works but large transfers hang | MTU: set 1450 on `vx0` and no more in the guest |
| "dropped inbound frames" warning in the log | frame rate above the queue; raise `packetQueueCapacity` |

## Security

- Tunnel frames are neither encrypted nor signed. Do not open the port to everyone: allow UDP only from the PC/server address. For public networks wrap the tunnel in a VPN.
- The hub accepts frames from any port of `remoteHost`; protection is the address and the random VNI.
- With the PC as a router the guest reaches everything the PC can see. Limit that with `forward` rules.
