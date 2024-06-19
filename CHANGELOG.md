# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.9](https://github.com/jbr/stopper/compare/v0.2.8...v0.2.9) - 2024-06-19

### Other
- *(deps)* update test-harness requirement from 0.2.0 to 0.3.0

## [0.2.8](https://github.com/jbr/stopper/compare/v0.2.7...v0.2.8) - 2024-05-30

### Fixed
- do not immediately register event listener

### Other
- add loom tests and enable miri
- ---
- *(deps)* bump codecov/codecov-action from 4.2.0 to 4.3.0
- *(deps)* bump codecov/codecov-action from 4.1.1 to 4.2.0
- *(deps)* bump actions/configure-pages from 4 to 5
- *(deps)* bump codecov/codecov-action from 4.1.0 to 4.1.1
- *(actions)* disable miri for now
- *(deps)* bump codecov/codecov-action from 4.0.1 to 4.1.0
- add coverage to readme
- *(actions)* move coverage job into ci workflow
- *(actions)* fix coverage workflow job name
- *(actions)* add coverage and use `cargo miri nextest run`

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
