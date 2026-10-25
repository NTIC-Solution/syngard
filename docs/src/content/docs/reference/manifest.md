---
title: Manifest format
description: The deterministic baseline format, syngard.manifest/v1.
---

A baseline is a JSON manifest. Its schema is published as
[`schemas/manifest.v1.schema.json`](https://github.com/NTIC-Solution/syngard/blob/main/schemas/manifest.v1.schema.json).

```json
{
  "digest": "0899d144…00e8",
  "format": "syngard.manifest/v1",
  "entries": [
    {
      "path": "SKILL.md",
      "kind": "instruction",
      "size": 195,
      "sha256": "1791790a…3b7a"
    }
  ]
}
```

## Rules

- `sha256` is the SHA-256 of the file's **raw bytes**: 64 lowercase hex
  characters. No normalization of line endings, whitespace, BOM or Unicode, so
  `\n` and `\r\n` produce different digests.
- `entries` are sorted strictly by path bytes. `path` is relative, `/`-separated,
  with no empty, `.` or `..` segment and no backslash.
- `digest` is the SHA-256 of the compact JSON serialization of
  `{"format": …, "entries": […]}`, fields in the order shown. Adding, removing,
  renaming or changing any entry changes it.
- Modification times are not part of the manifest: they are not evidence of
  integrity.

## Loading

When syngard loads a baseline it recomputes the digest. A mismatch, unsorted
entries, an unknown `format` or an invalid path rejects the file with exit 4.
A tampered baseline is never repaired.
