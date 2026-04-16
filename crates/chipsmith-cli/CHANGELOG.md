# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/cramt/chipsmith/releases/tag/chipsmith-cli-v0.1.0) - 2026-04-16

### Added

- add `chipsmith init` and `chipsmith cables` commands
- add `chipsmith flash` command to program FPGA via JTAG
- initial commit as chipsmith

### Other

- extract shared code into chipsmith-quartus-common, make Toolchain trait object-safe
- cargo fmt
