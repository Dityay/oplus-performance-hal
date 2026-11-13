#!/usr/bin/env python3
"""Generate a full-coverage IPC probe for the reconstructed performance HAL.

One call per interface method, so "does the HAL actually return data on this
device?" is measured over all 169 methods instead of sampled.

Reads the method list from the generated AIDL header, emits tools/ipc_probe_full.cpp.
"""
import re, sys

HDR = sys.argv[1]
OUT = sys.argv[2]

src = open(HDR).read()
# the pure-virtual declarations in the IPerformance interface
decls = re.findall(
    r"virtual ::ndk::ScopedAStatus\s+([A-Za-z_0-9]+)\(([^;]*?)\)\s*=\s*0;",
    src, re.S)
if not decls:
    sys.exit("no method declarations found in " + HDR)

STRUCTS = {"ProcReqHal", "ProcMemStatRet", "OsvelteVersionRet"}

# Methods that mutate device state. The full probe must NOT call these with
# dummy arguments: setFreqGoverner("") would write an empty governor,
# setClmThreshold("", 0) would write 0 into the CPU load threshold, setImFlag
# would clear the important-task list. They are skipped unless the probe is
# explicitly invoked with --with-writes.
MUTATING_PREFIXES = ("write", "set", "enable", "disable", "add", "del",
                     "remove", "clear", "perProcess")

def is_mutating(name):
    if name.startswith(MUTATING_PREFIXES):
        return True
    if name.startswith("hybridswap_") and name.endswith("_write"):
        return True
    return False

def split_args(a):
    out, depth, cur = [], 0, ""
    for ch in a:
        if ch == "<":
            depth += 1
        elif ch == ">":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out

def parse(t):
    t = t.strip()
    m = re.match(r"^(.*?)([A-Za-z_0-9:]+)$", t.replace("&", "& "), re.S)
    # simpler: split off the trailing name
    toks = t.replace("&", " & ").split()
    name = toks[-1]
    typ = " ".join(toks[:-1]).replace(" & ", "&")
    return typ, name

blocks = []
calls = []
for name, argstr in decls:
    args = split_args(argstr)
    call_args = []
    decls_out = []
    out_kind = None
    out_name = None
    for a in args:
        typ, an = parse(a)
        is_ptr = "*" in typ
        base = typ.replace("const ", "").replace("&", "").replace("*", "").strip()
        base = base.lstrip(":")
        if not base.startswith("std::"):
            base = base.split("::")[-1]
        if is_ptr:
            # out param
            if base == "std::string":
                decls_out.append(f'        std::string {an};')
                out_kind, out_name = "str", an
                call_args.append(f"&{an}")
            elif base in ("int32_t", "int64_t"):
                decls_out.append(f'        {base} {an} = -424242;')
                out_kind, out_name = "int", an
                call_args.append(f"&{an}")
            elif base == "bool":
                decls_out.append(f'        bool {an} = false;')
                out_kind, out_name = "bool", an
                call_args.append(f"&{an}")
            else:
                decls_out.append(f'        {base} {an};')
                out_kind, out_name = "struct", an
                call_args.append(f"&{an}")
        else:
            if base == "std::string":
                if name in ("getExt4FragScore", "getExt4FreefragInfo"):
                    # the path is a template; pass a real ext4 device name
                    decls_out.append(f'        std::string {an} = "dm-11";')
                else:
                    decls_out.append(f'        std::string {an} = "";')
                call_args.append(an)
            elif base.startswith("std::vector"):
                if name == "readMemoryByPids":
                    # pass a real pid so the smaps_rollup path is exercised
                    decls_out.append(f'        std::vector<int> {an} = {{ (int)getpid() }};')
                elif name == "readMemoryByUids":
                    # pass a real uid so the per-uid walk is exercised
                    decls_out.append(f'        std::vector<int> {an} = {{ 1000 }};')
                else:
                    decls_out.append(f'        std::vector<int> {an};')
                call_args.append(an)
            elif base == "bool":
                decls_out.append(f'        bool {an} = false;')
                call_args.append(an)
            elif base in ("int32_t", "int64_t"):
                decls_out.append(f'        {base} {an} = 0;')
                call_args.append(an)
            elif base in STRUCTS:
                decls_out.append(f'        {base} {an};')
                call_args.append(an)
            else:
                decls_out.append(f'        {base} {an} = {{}};')
                call_args.append(an)

    body = "\n".join(decls_out)
    call = f'        auto st = svc->{name}({", ".join(call_args)});'
    if out_kind == "str":
        rep = f'        P("  {name} -> %zu bytes", {out_name}.size());'
    elif out_kind == "int":
        rep = f'        P("  {name} -> %lld", (long long){out_name});'
    elif out_kind == "bool":
        rep = f'        P("  {name} -> %d", (int){out_name});'
    else:
        if out_kind is None:
            rep = f'        P("  {name} -> (void, no out-param)");'
        elif base == "ProcReqHal":
            rep = (f'        P("  {name} -> reqId=%d stats=%zu", {out_name}.reqId, '
                   f'{out_name}.stats.size());')
        elif base == "OsvelteVersionRet":
            rep = (f'        P("  {name} -> v=%d major=%d minor=%d patch=%d", '
                   f'{out_name}.version, {out_name}.major, {out_name}.minor, {out_name}.patch);')
        else:
            rep = (f'        P("  {name} -> status=%d count=%d", {out_name}.status, '
                   f'{out_name}.count);\n'
                   f'        if ({out_name}.count > 0) P("      first: pid=%d rss=%d pss=%d comm=%s", '
                   f'{out_name}.stats[0].pid, {out_name}.stats[0].rss, {out_name}.stats[0].pss, '
                   f'{out_name}.stats[0].comm.c_str());')
    if is_mutating(name):
        blk = ("    if (with_writes) {\n" + body + "\n" + call +
               f'\n        note("{name}", (int)st.getExceptionCode());\n' + rep +
               "\n    } else { skipped++; }")
    else:
        blk = ("    {\n" + body + "\n" + call +
               f'\n        note("{name}", (int)st.getExceptionCode());\n' + rep + "\n    }")
    blocks.append(blk)
    calls.append(name)

prog = f'''// GENERATED by tools/gen_probe_full.py - do not edit by hand.
// One call per IPerformance method, so coverage over all {len(calls)} methods is
// measured on the device instead of sampled.
#include <android/binder_manager.h>
#include <android/binder_process.h>
#include <cstdio>
#include <string>
#include <vector>
#include <unistd.h>
#include <aidl/vendor/oplus/hardware/performance/IPerformance.h>
#include <aidl/vendor/oplus/hardware/performance/ProcReqHal.h>
#include <aidl/vendor/oplus/hardware/performance/ProcMemStatRet.h>
#include <aidl/vendor/oplus/hardware/performance/OsvelteVersionRet.h>

using aidl::vendor::oplus::hardware::performance::IPerformance;
using aidl::vendor::oplus::hardware::performance::ProcReqHal;
using aidl::vendor::oplus::hardware::performance::ProcMemStatRet;
using aidl::vendor::oplus::hardware::performance::OsvelteVersionRet;

static const char* kName = "vendor.oplus.hardware.performance.IPerformance/default";
#define P(fmt, ...) printf(fmt "\\n", ##__VA_ARGS__)

int main(int argc, char** argv) {{
    ABinderProcess_startThreadPool();
    AIBinder* raw = AServiceManager_checkService(kName);
    if (raw == nullptr) {{
        P("FAIL: %s not found", kName);
        return 2;
    }}
    ::ndk::SpAIBinder binder(raw);
    auto svc = IPerformance::fromBinder(binder);
    if (svc == nullptr) {{ P("FAIL: fromBinder null"); return 2; }}

    int fails = 0;
    int skipped = 0;
    // Mutating methods are skipped unless --with-writes is passed: calling them
    // with dummy arguments would change device state (governor, CPU load
    // threshold, important-task list).
    bool with_writes = (argc > 1 && strcmp(argv[1], "--with-writes") == 0);
    auto note = [&](const char* what, int code) {{
        if (code != 0) {{ P("  EXCEPTION %-30s code=%d", what, code); ++fails; }}
    }};

{chr(10).join(blocks)}

    P("SUMMARY methods={len(calls)} skipped_mutating=%d exceptions=%d", skipped, fails);
    return 0;
}}
'''
open(OUT, "w").write(prog)
print(f"wrote {OUT}: {len(calls)} method calls")
