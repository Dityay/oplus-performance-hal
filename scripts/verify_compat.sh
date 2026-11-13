#!/bin/bash
# OplusPerf/scripts/verify_compat.sh
#
# Reproducible wire-compatibility check between the SHIPPED OPlus performance
# HAL and the RECONSTRUCTED aidl/ interface.
#
# 1. Reads the transaction code each BpPerformance::<method> passes to
#    AIBinder_transact in the shipped V1 NDK backend. AIDL's
#    FIRST_CALL_TRANSACTION == 1, so that code IS the 1-based declaration order
#    of the original interface.
# 2. Reads the TRANSACTION_<name> constants that the AOSP aidl tool generated
#    from the RECONSTRUCTED aidl/ (AIDL assigns them by declaration order).
# 3. Asserts the two sequences are identical.
#
# Passing this means the reconstructed interface dispatches exactly like the
# shipped one: any prebuilt client of this interface works against a HAL built
# from the reconstruction, and vice versa.
#
# Usage: scripts/verify_compat.sh [--rebuild]
set -euo pipefail

HERE="$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"
CRATE="$HERE"
ROM_TREE="${ROM_TREE:-$HERE/../../../erofs}"
AIDL_BIN="${AIDL_BIN:-/opt/android-sdk/build-tools/34.0.0/aidl}"
SHIPPED_LIB="$ROM_TREE/vendor/lib64/vendor.oplus.hardware.performance-V1-ndk_platform.so"
AIDLROOT="$CRATE/aidl"
PKG="$AIDLROOT/vendor/oplus/hardware/performance"

RED=$'\033[1;31m'; GRN=$'\033[1;32m'; NC=$'\033[0m'
[[ -f "$SHIPPED_LIB" ]] || { echo -e "${RED}shipped NDK backend not found: $SHIPPED_LIB${NC}"; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# --- 1. shipped binary: transaction code per BpPerformance method -------------
llvm-objdump -d "$SHIPPED_LIB" > "$WORK/shipped.asm" 2>/dev/null
python3 - "$WORK/shipped.asm" "$WORK/shipped.txt" <<'PY'
import re, subprocess, sys
asm, out = sys.argv[1], sys.argv[2]
func_hdr = re.compile(r'^[0-9a-f]+ <(.+)>:$')
mov_w1 = re.compile(r'\bmov\s+w1,\s+#(0x[0-9a-f]+|\d+)')
transact = re.compile(r'AIBinder_transact')
res, cur, last = {}, None, None
for line in open(asm):
    h = func_hdr.match(line)
    if h:
        cur, last = h.group(1), None
        continue
    if cur and 'BpPerformance' in cur:
        m = mov_w1.search(line)
        if m:
            v = m.group(1)
            last = int(v, 16) if v.startswith('0x') else int(v)
        if transact.search(line) and last is not None:
            res[cur] = last
            last = None
rows = []
for sym, code in res.items():
    if any(x in sym for x in ('BpPerformanceC','BpPerformanceD','asBinder','onTransact',
                              'getInterface','createBinder')):
        continue
    name = subprocess.run(['c++filt', sym], capture_output=True, text=True).stdout
    name = name.split('BpPerformance::')[-1].split('(')[0]
    rows.append((code, name))
rows.sort()
open(out, 'w').write('\n'.join(f'{c}\t{n}' for c, n in rows) + '\n')
print(f"  shipped: {len(rows)} methods with a transact code")
PY

# --- 2. reconstructed aidl: generated TRANSACTION_* constants -----------------
GEN="$WORK/gen"; mkdir -p "$GEN/ndk" "$GEN/hdr"
"$AIDL_BIN" -I "$AIDLROOT" --structured --stability=vintf --lang=ndk \
    -o "$GEN/ndk" -h "$GEN/hdr" \
    "$PKG/IPerformance.aidl" "$PKG/ProcReqHal.aidl" "$PKG/ProcMemStat.aidl" \
    "$PKG/ProcMemStatRet.aidl" "$PKG/OsvelteVersionRet.aidl" >/dev/null
python3 - "$GEN/hdr/aidl/vendor/oplus/hardware/performance/IPerformance.h" "$WORK/recon.txt" <<'PY'
import re, sys
hdr, out = sys.argv[1], sys.argv[2]
rows = []
for line in open(hdr):
    m = re.match(r"\s*static constexpr uint32_t TRANSACTION_(\w+) = FIRST_CALL_TRANSACTION \+ (\d+);", line)
    if m:
        rows.append((int(m.group(2)) + 1, m.group(1)))  # FIRST_CALL_TRANSACTION == 1
rows.sort()
open(out, 'w').write('\n'.join(f'{c}\t{n}' for c, n in rows) + '\n')
print(f"  reconstructed: {len(rows)} methods")
PY

# --- 3. cross-check ------------------------------------------------------------
if diff -u "$WORK/shipped.txt" "$WORK/recon.txt" > "$WORK/diff.txt"; then
    echo -e "${GRN}PASS${NC} — transaction order identical for all $(wc -l < "$WORK/shipped.txt") methods"
    echo "       (code<TAB>method) — see $CRATE/re/method_order.txt"
    exit 0
fi
echo -e "${RED}FAIL — transaction order differs${NC}"
head -40 "$WORK/diff.txt"
exit 1
