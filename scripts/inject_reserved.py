#!/usr/bin/env python3
"""Inject the AIDL reserved transactions (getInterfaceHash / getInterfaceVersion)
into an aidl-generated onTransact switch.

Modern `aidl` only emits the reserved transactions for frozen interfaces, but
the shipped OnePlus vendor backends were built with an older aidl that did
emit them, and their clients may call them. Without them the service answers
STATUS_UNKNOWN_TRANSACTION to transaction codes 16777214 / 16777215.

The values are taken verbatim from the shipped library:
  getInterfaceVersion -> 1              (BnPerformance::getInterfaceVersion:
                                          mov w8, #0x1)
  getInterfaceHash    -> <40 hex chars> (IPerformance::hash)

The wire encoding matches every other out-parameter in the generated code:
AParcel_writeStatusHeader(OK) followed by AParcel_writeData(out_param).
"""
import re, sys

src, hashval = sys.argv[1], sys.argv[2]

with open(src) as f:
    text = f.read()

if "16777214" in text:
    print(f"  {src}: reserved transactions already present")
    sys.exit(0)

anchor = "case (FIRST_CALL_TRANSACTION + 0 "
idx = text.find(anchor)
if idx < 0:
    sys.exit(f"ERROR: no dispatch anchor found in {src}")

# Insert before the first dispatch case, inside the switch.
inject = f"""    case 16777214 /*getInterfaceHash*/: {{
      std::string _aidl_return = "{hashval}";
      _aidl_ret_status = AParcel_writeStatusHeader(_aidl_out, ::ndk::AStatus::OK());
      if (_aidl_ret_status == STATUS_OK) {{
        _aidl_ret_status = ::ndk::AParcel_writeData(_aidl_out, _aidl_return);
      }}
      break;
    }}
    case 16777215 /*getInterfaceVersion*/: {{
      int32_t _aidl_return = 1;
      _aidl_ret_status = AParcel_writeStatusHeader(_aidl_out, ::ndk::AStatus::OK());
      if (_aidl_ret_status == STATUS_OK) {{
        _aidl_ret_status = ::ndk::AParcel_writeData(_aidl_out, _aidl_return);
      }}
      break;
    }}
"""

text = text[:idx] + inject + text[idx:]
with open(src, "w") as f:
    f.write(text)
print(f"  {src}: injected reserved transactions (hash={hashval})")
