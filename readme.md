# Obsidian Sync Tool

This tool synchronizes two obsidian vaults based on yaml metadata.

The idea is that notes in two different obisidan vaults will be kept in sync if they have specific metadata to do so.

```yaml
---
sync: true
---
```

# Testing

Test cases:
- Note added
- Note renamed
- Note modified
- Note deleted