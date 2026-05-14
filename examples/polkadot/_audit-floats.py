"""Internal helper for `audit-floats.sh`.

Reads `wasm-tools print` output from stdin, writes a per-runtime TSV (one
record per f32/f64 op occurrence) and a markdown summary (function-level
rollup) to the paths given on argv.
"""

import re
import sys
from collections import Counter, defaultdict


def main() -> None:
    short, tsv_path, md_path = sys.argv[1:4]

    # `(func $name (;idx;)` — wasm-tools format.
    fn_re = re.compile(r"^\s*\(func\s+\$(\S+)\s+\(;(\d+);\)")
    # Any f32/f64 op variant.
    op_re = re.compile(r"\b((?:f32|f64)\.[a-z_0-9]+)\b")

    # Best-effort Rust symbol cleaner that tolerates the wasm-tools `$LT$` /
    # `$RF$` / `..` escapes the WAT printer applies to keep names valid
    # identifiers. Not a full demangler — but reads.
    escape_map = [
        ("$LT$", "<"),
        ("$GT$", ">"),
        ("$RF$", "&"),
        ("$BP$", "*"),
        ("$u20$", " "),
        ("$u27$", "'"),
        ("$u7b$", "{"),
        ("$u7d$", "}"),
        ("$u7e$", "~"),
        ("..", "::"),
    ]

    def unescape(s: str) -> str:
        for k, v in escape_map:
            s = s.replace(k, v)
        return s

    def short_name(mangled: str) -> str:
        # Legacy Rust mangle: `_ZN<len><name><len><name>...17h<hash>E` where
        # length counts are against the original wasm identifier (with
        # `$LT$`/`$RF$` escapes still in place). So we must split first,
        # *then* unescape each segment — otherwise `$LT$` collapses to `<`
        # and the length prefixes go wrong.
        n = re.sub(r"^_ZN", "", mangled)
        n = re.sub(r"17h[a-f0-9]+E$", "", n)

        out: list[str] = []
        i = 0
        while i < len(n):
            m = re.match(r"\d+", n[i:])
            if not m:
                # Not a length prefix — append remainder verbatim and stop.
                if n[i:]:
                    out.append(unescape(n[i:]))
                break
            length = int(m.group(0))
            i += len(m.group(0))
            seg = n[i : i + length]
            i += length
            out.append(unescape(seg))
        return "::".join(s for s in out if s)

    records: list[tuple[int, str, str, str]] = []
    fname, fidx = "", ""
    for line in sys.stdin:
        m = fn_re.match(line)
        if m:
            fname, fidx = m.group(1), m.group(2)
            continue
        m = op_re.search(line)
        if m and fname:
            records.append((int(fidx), m.group(1), fname, short_name(fname)))

    with open(tsv_path, "w") as f:
        f.write("fn_idx\top\tname_raw\tname_short\n")
        for r in records:
            f.write("\t".join(str(x) for x in r) + "\n")

    by_fn: dict[tuple[int, str], Counter[str]] = defaultdict(Counter)
    for fidx_n, op, _raw, short_n in records:
        by_fn[(fidx_n, short_n)][op] += 1

    fn_total = {k: sum(v.values()) for k, v in by_fn.items()}

    with open(md_path, "w") as f:
        f.write(f"# Float-op locations — `{short}`\n\n")
        f.write(
            f"**Total float-op occurrences:** {len(records)} across "
            f"**{len(by_fn)} function(s)**.\n\n"
        )
        f.write(
            "Each row is one function that contains float operations, sorted "
            "by op count. Op kinds are listed with multiplicity (e.g. "
            "`f64.load`×3 means three `f64.load` instructions in this "
            "function).\n\n"
        )
        f.write("| Fn idx | Function | Total | Op kinds |\n")
        f.write("|-------:|----------|------:|----------|\n")
        for (fidx_n, short_n), ops in sorted(
            by_fn.items(),
            key=lambda kv: (-fn_total[kv[0]], kv[0][0]),
        ):
            kind_str = ", ".join(
                f"`{k}`×{c}" if c > 1 else f"`{k}`" for k, c in ops.most_common()
            )
            safe_name = short_n.replace("|", "\\|")
            f.write(
                f"| {fidx_n} | `{safe_name}` | {ops.total()} | {kind_str} |\n"
            )


if __name__ == "__main__":
    main()
