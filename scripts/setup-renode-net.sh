#!/bin/bash
# =============================================================================
# setup-renode-net.sh
# Configura la rete host per l'emulazione Renode/Wasmbed.
# Va eseguito DOPO che Renode ha creato l'interfaccia tap0.
#
# Topologia:
#   Zephyr (Renode) ←→ tap0 (192.168.1.1/24) ←→ [MASQUERADE] ←→ cni0 (k3s pods 10.42.x.x)
#
# Il firmware si connette al gateway via ClusterIP (10.43.142.48:8081).
# kube-proxy DNAT in PREROUTING mappa ClusterIP → pod IP (10.42.0.x:8443).
# MASQUERADE su cni0 riscrive l'IP sorgente (192.168.1.x → 10.42.0.1) così
# la risposta dal pod torna al host, che la rigira via conntrack a tap0.
#
# Uso: sudo ./scripts/setup-renode-net.sh
# =============================================================================

set -e

TAP_IP="192.168.1.1"
DEVICE_SUBNET="192.168.1.0/24"
K3S_IFACE="cni0"    # bridge k3s (pod network 10.42.0.0/16)
WAN_IFACE="ens18"   # interfaccia fisica

echo "=== Wasmbed Renode network setup ==="

# 1. Verifica che tap0 esista
if ! ip link show tap0 &>/dev/null; then
  echo "ERRORE: tap0 non trovata. Avvia prima Renode (Connect dal API/Dashboard)."
  exit 1
fi

# 2. Configura tap0
ip addr flush dev tap0 2>/dev/null || true
ip addr add ${TAP_IP}/24 dev tap0
ip link set tap0 up
echo "✅ tap0: ${TAP_IP}/24"

# 3. IP forwarding
sysctl -w net.ipv4.ip_forward=1 > /dev/null
echo "✅ ip_forward abilitato"

# 4. Regole iptables forwarding tap0 ↔ k3s bridge (cni0)
iptables -C FORWARD -i tap0 -o ${K3S_IFACE} -j ACCEPT 2>/dev/null || \
  iptables -I FORWARD -i tap0 -o ${K3S_IFACE} -j ACCEPT
iptables -C FORWARD -i ${K3S_IFACE} -o tap0 -m state --state ESTABLISHED,RELATED -j ACCEPT 2>/dev/null || \
  iptables -I FORWARD -i ${K3S_IFACE} -o tap0 -m state --state ESTABLISHED,RELATED -j ACCEPT
# Permetti anche tap0 → ens18 (per NodePort su IP host)
iptables -C FORWARD -i tap0 -o ${WAN_IFACE} -j ACCEPT 2>/dev/null || \
  iptables -I FORWARD -i tap0 -o ${WAN_IFACE} -j ACCEPT
iptables -C FORWARD -i ${WAN_IFACE} -o tap0 -m state --state ESTABLISHED,RELATED -j ACCEPT 2>/dev/null || \
  iptables -I FORWARD -i ${WAN_IFACE} -o tap0 -m state --state ESTABLISHED,RELATED -j ACCEPT
echo "✅ iptables: forwarding tap0 ↔ ${K3S_IFACE} e ${WAN_IFACE}"

# 5a. MASQUERADE per traffico da device verso pod K3s (via cni0)
#     Necessario: il pod risponde a 10.42.0.1 (cni0 IP) non a 192.168.1.x
iptables -t nat -C POSTROUTING -s ${DEVICE_SUBNET} -o ${K3S_IFACE} -j MASQUERADE 2>/dev/null || \
  iptables -t nat -A POSTROUTING -s ${DEVICE_SUBNET} -o ${K3S_IFACE} -j MASQUERADE
echo "✅ NAT masquerade: ${DEVICE_SUBNET} → ${K3S_IFACE} (pod network)"

# 5b. MASQUERADE per traffico da device verso rete esterna (via ens18)
iptables -t nat -C POSTROUTING -s ${DEVICE_SUBNET} -o ${WAN_IFACE} -j MASQUERADE 2>/dev/null || \
  iptables -t nat -A POSTROUTING -s ${DEVICE_SUBNET} -o ${WAN_IFACE} -j MASQUERADE
echo "✅ NAT masquerade: ${DEVICE_SUBNET} → ${WAN_IFACE}"

# 6. Avvia dnsmasq per DHCP su tap0
pkill dnsmasq 2>/dev/null || true
sleep 1
dnsmasq --conf-file=/etc/dnsmasq.d/wasmbed-tap.conf --pid-file=/tmp/wasmbed-dnsmasq.pid
echo "✅ dnsmasq DHCP avviato (192.168.1.100-200, GW=192.168.1.1)"

GATEWAY_TLS="10.43.142.48:8081"
echo ""
echo "=== Rete pronta ==="
echo "    Device riceverà IP: 192.168.1.100-200"
echo "    Gateway TLS (ClusterIP): ${GATEWAY_TLS}"
echo "    Test connettività: nc -zv ${GATEWAY_TLS%%:*} ${GATEWAY_TLS##*:}"
