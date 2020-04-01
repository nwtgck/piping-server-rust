# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

## [Unreleased]

## [0.5.3] - 2020-04-01
### Changed
* Update dependencies

## [0.5.2] - 2020-03-21
### Changed
* Update dependencies

## [0.5.1] - 2020-03-13
### Changed
* Use stable, not nightly

## [0.5.0] - 2020-03-12
### Changed
* Update dependencies
* Use new library features including async/await (only dev)

## [0.4.9] - 2019-12-22
### Changed
* Update dependencies

## [0.4.8] - 2019-11-08
### Changed
* Update simple Web UI

## [0.4.7] - 2019-10-16
### Changed
* Update dependencies

## [0.4.6] - 2019-09-29
### Changed
* Update dependency

## [0.4.5] - 2019-09-14
### Changed
* Update dependency

## [0.4.4] - 2019-09-08
### Changed
* Update dependencies

## [0.4.3] - 2019-09-05
### Changed
* Update dependencies

## [0.4.2] - 2019-08-30
### Changed
* Update dependencies

## [0.4.1] - 2019-08-16
### Fixed
* Specify `"stack": "container"` in app.json

## [0.4.0] - 2019-08-16
### Added
* Add simple Web UI
* Add `/version` routing
* Support Heroku

## [0.3.0] - 2019-08-03
### Changed
* Allow cross-origin
* Pass sender's Content-Type, Content-Length and Content-Disposition headers to receiver
* Support Preflight request

## [0.2.2] - 2019-07-28
### Changed
* Generalize ReceiverResBody as FinishDetectableBody

## [0.2.1] - 2019-07-28
### Fixed
* Not close sender's connection when transferring throw Caddy reverse proxy

## [0.2.0] - 2019-07-27
### Changed
* Use req-res handler like Node.js
* Return non-2xx status codes when rejecting

## 0.1.0 - 2019-07-17
### Added
* Implement basic Piping Server

[Unreleased]: https://github.com/nwtgck/piping-server-rust/compare/v0.5.3...HEAD
[0.5.3]: https://github.com/nwtgck/piping-server-rust/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.9...v0.5.0
[0.4.9]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.8...v0.4.9
[0.4.8]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.7...v0.4.8
[0.4.7]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.6...v0.4.7
[0.4.6]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.1.0...v0.2.0
