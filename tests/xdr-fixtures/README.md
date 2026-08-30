# XDR conformance fixtures (issue #1516)

Golden fixtures shared by the XDR serialization conformance suite
(`tests/xdr_conformance.rs`) and, once merged, the M1 trace decoder in `libs`
so both sides verify the **same** encoding contract from
`docs/SERIALIZATION_STANDARDS.md`.

| File | Purpose |
|---|---|
| `option_none.xdr` | Canonical XDR for `Option::<u32>::None` → `SCVal::Void` (`00 00 00 01`, discriminant `SCV_VOID = 1`). |
| `option_some_u32_max.xdr` | Canonical XDR for `Option::<u32>::Some(u32::MAX)` → `SCVal::U32` (`00 00 00 03 ff ff ff ff`, discriminant `SCV_U32 = 3`). Proves `None` never collides with a sentinel `Some(value)`. |
| `scval-mapping.json` | Ordered table of every canonical type named in `docs/SERIALIZATION_STANDARDS.md`: `option` (`Void`/`Some`), `bool`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `bytes`, `string`, `symbol`, `vec`, `map`. For each entry the `expected_json` column is the typed JSON the decoder must emit for the representative value; the conformance suite asserts the round-trip encoding of that value is byte-stable and maps to exactly that JSON. |

The decoder (`libs`) must load `scval-mapping.json` and the `.xdr` binaries
from this directory, not its own copy, so a change to the encoding contract
fails CI here before the decoder can drift.

Round-trip values asserted by the suite:

- `option_none` → `None`, 4 bytes (`00 00 00 01`)
- `option_some` → `Some(42u32)`, `SCVal::U32`
- `bool` → `true`
- `u32` → `u32::MAX`
- `i32` → `i32::MIN`
- `u64` → `u64::MAX`
- `i64` → `i64::MIN`
- `u128` → `u128::MAX`
- `i128` → `i128::MIN`
- `bytes` → `[0xf0, 0x0d, 0xca, 0xfe]`
- `string` → `"herbal therapy"`
- `symbol` → `"STATE_LEDGER"`
- `vec` → `[1, 2, 3]`
- `map` → `{ "alpha": 1, "beta": 2 }`

Run: `cargo test --manifest-path tests/Cargo.toml --test xdr_conformance`
(wired into CI via `.github/workflows/xdr-conformance.yml`).