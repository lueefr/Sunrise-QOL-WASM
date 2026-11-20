# Full module coverage ledger

Run `python3 tools/parity/report.py --output target/parity-coverage.json` to
rebuild the machine-readable status of **all 44 configured modules**. The
catalog cross-checks the interactive GUI against the source-parity matrix and
rejects newly introduced modules with no explicit parity row or limitations.
Its report distinguishes an inspected upstream source from runtime testing;
neither independently establishes 1:1 parity.

The requested Full Brightness Toggle is a separately tracked, not-configured
feature because the public WASM lightmap/gamma API is absent. Tree Harvester
uses the shared Leaf Decay module rather than duplicating that logic.

**Certification requires original-vs-ported differential in-game outcomes for
all significant configurations, entity types, event order, RNG and edge cases.**
The current CI has only one attempted real-engine scenario group and requires
private licensed Minecraft resources on an authorized runner. No feature is
claimed certified on the strength of synthetic-host or source-only tests.

Both CI workflows now attach a `sunrise-qol-parity-matrix` artifact. It
lists every module, whether its original source was inspected, source-status,
known differences and explicit full-1:1 certification=false. This artifact
is independent of the real-engine gameplay evidence and does not replace it.
