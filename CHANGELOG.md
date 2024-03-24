# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.7](https://github.com/jbr/stopper/compare/v0.2.6...v0.2.7) - 2024-03-24

### Fixed
- correct usage of EventListener for scenarios that involve multiple wakes

### Other
- *(deps)* bump actions/checkout from 3 to 4
- update readme and docs
- fix edition

## [0.2.6](https://github.com/jbr/stopper/compare/v0.2.5...v0.2.6) - 2024-03-08

### Fixed
- correctly delegate Stream::size_hint to the inner Stream

## [0.2.5](https://github.com/jbr/stopper/compare/v0.2.4...v0.2.5) - 2024-03-07

### Other
- *(actions)* add miri test
- *(deps)* upgrade to event-listener 5
- *(legal)* add licenses
- *(actions)* add dependabot
- *(actions)* fix docs workflow
- *(actions)* use rust-cache in ci.yml

## [0.2.4](https://github.com/jbr/stopper/compare/v0.2.3...v0.2.4) - 2024-01-23

### Added
- stopper can be awaited

### Other
- use release-plz
