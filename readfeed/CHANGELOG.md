# CHANGELOG

### [Unreleased]

## [0.2.0] - 2023-12-18

### Added

* Update `maybe_xml` dependency to version `0.11.0.`

## [0.1.3] - 2023-11-29

### Added

* Add `.tag_name()` for `Unknown` types to get the unknown element name.

### Fixed

* Fix bug in detecting matching end tag. When namespaces were used, the end tag
  would not be found.

## [0.1.2] - 2023-11-26

### Fixed

* Fix reading of content when the same tag name is nested.

## [0.1.1] - 2023-11-26

### Added

* OPML reader.

## [0.1.0] - 2023-11-21

### Added

* Initial implementation.

[Unreleased]: https://github.com/bluk/readfeed/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/bluk/readfeed/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/bluk/readfeed/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/bluk/readfeed/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/bluk/readfeed/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/bluk/readfeed/releases/tag/v0.1.0