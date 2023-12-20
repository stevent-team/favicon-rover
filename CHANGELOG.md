# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/stevent-team/favicon-rover/compare/v0.1.0...v0.1.1) - 2023-12-20

### Other
- Add an important comment
- Exclude fallback code without server feature
- Run CI with all features
- Send cache control header
- Use EnvFilter for tracing
- Implement CORS protection
- Render SVG favicons
- Convert to image format based on Accept header
- Refactor favicon responses
- Generate fallback initial code
- Add webp codec support
- Create FaviconImage wrapper for image
- Setup cargo feature for server
- Implement writing images to stdout and files
- Wrap image structs in custom Image struct
- Correctly call .await on fallback
- Correctly call .await on fallback
- Continue working on fetching favicons
- Add a scrape_link_tags fn
