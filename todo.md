# Future Work & Post-v2 Improvements

This document tracks potential improvements and features for future releases beyond v2.0.0.

**Status**: v2.0.0 complete - Core features implemented and stable

---

## Integration Testing

- [ ] Integration tests for persistence layer
- [ ] End-to-end tests for spell execution
- [ ] Automated UI testing with simulated events

## Input Popup Integration

- [ ] Trigger InputPopup when executing spells with placeholders (e.g., `<pid>`, `<port>`)
- [ ] Wire up placeholder substitution before execution
- [ ] Document placeholder syntax for users

## Platform Support

- [ ] Windows compatibility (exec() fallback)
- [ ] macOS-specific optimizations
- [ ] Test on various terminal emulators

## Advanced Features (Future Versions)

- [ ] Spell tagging/categories beyond glyphs
- [ ] Spell search by tags
- [ ] Custom themes (user-defined colors)
- [ ] Plugin system for custom spell types
- [ ] Spell sharing via URLs
- [ ] Cloud sync for spellbooks

## Performance Optimizations

- [ ] Virtual scrolling for large spellbooks (>1000 spells)
- [ ] Lazy loading of spell details
- [ ] Optimize search for large codices

## Documentation

- [ ] Video tutorials for new users
- [ ] Example spellbook collections
- [ ] Migration guide from other tools

---

**Last Updated**: 2026-03-25

For current status and testing, see [AGENTS.md](AGENTS.md)
For architecture details, see [docs/architecture.md](docs/architecture.md)
