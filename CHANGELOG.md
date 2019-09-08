# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

## [Unreleased]

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

[Unreleased]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.4...HEAD
[0.4.4]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.1.0...v0.2.0
