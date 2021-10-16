# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

## [Unreleased]

## [0.9.1] - 2021-10-16
### Changed
* Update dependencies
* (Docker) Use Rust version 1.55.0 in build

## [0.9.0] - 2021-10-13
### Changed
* Update dependencies
* Reject POST and PUT with Content-Range for now to detect resumable upload in the future
* Add X-Robots-Tag: "none" header to receiver's response
* (Docker) Allow Docker users to run without --init
* Respond 405 Method Not Allowed when method is not supported
* Support HEAD method for the reserved paths

### Added
* Support multipart upload
* Create /noscript Web UI for transferring a file without JavaScript
* Support `X-Piping` header passing arbitrary data from sender to receivers
* Add /help page

## [0.8.5] - 2021-07-24
### Changed
* Update dependencies

## [0.8.4] - 2021-07-18
### Changed
* Update dependencies

## [0.8.3] - 2021-05-29
### Changed
* Update dependencies

## [0.8.2] - 2021-01-12
### Changed
* (internal) Update dependencies and update codes for the updates

## [0.8.1] - 2020-09-19
### Changed
* (internal) Use pin-project-lite for removing Box from FinishDetectableStream
* (internal) Not use `tokio::spawn()`
* Update dependencies

## [0.8.0] - 2020-09-06
### Added
* Send messages to sender before transferring

## [0.7.2] - 2020-09-01
### Changed
* Simplify links in Web UI

## [0.7.1] - 2020-09-01
### Changed
* (internal) Improve implementation
* Update dependencies

## [0.7.0] - 2020-08-30
### Changed
* Reject reserved path sending
* Return 204 at /favicon.ico
* Return 404 at /robots.txt

## [0.6.2] - 2020-08-29
### Changed
* (Docker) Minimize Docker image
* Reject Service Worker registration

## [0.6.1] - 2020-08-26
### Changed
* Set default log level as INFO
* Add init support for Docker image

## [0.6.0] - 2020-08-25
### Changed
* Update dependencies

### Added
* Support HTTPS
* Support logging with date

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

[Unreleased]: https://github.com/nwtgck/piping-server-rust/compare/v0.9.1...HEAD
[0.9.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.8.5...v0.9.0
[0.8.5]: https://github.com/nwtgck/piping-server-rust/compare/v0.8.4...v0.8.5
[0.8.4]: https://github.com/nwtgck/piping-server-rust/compare/v0.8.3...v0.8.4
[0.8.3]: https://github.com/nwtgck/piping-server-rust/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.6.2...v0.7.0
[0.6.2]: https://github.com/nwtgck/piping-server-rust/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/nwtgck/piping-server-rust/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/nwtgck/piping-server-rust/compare/v0.5.3...v0.6.0
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
