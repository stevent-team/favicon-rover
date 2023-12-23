# ☄️ Favicon Rover

[![crates.io version](https://img.shields.io/crates/v/favicon-rover)](https://crates.io/crates/favicon-rover)
[![docs.rs](https://img.shields.io/docsrs/favicon-rover)](https://docs.rs/crate/favicon-rover/latest)

Fetch the favicon of any website.

- ⌨️ CLI tool
- 🌐 Web server
- 🛟 Fallback icons
- 🦀 Rust

## Install

```bash
cargo install favicon-rover
```

## CLI Usage

Fetch the favicon for a site using the cli tool

```bash
# Usage: favicon-rover get [OPTIONS] <URL>

favicon-rover get https://crates.io # output the crates favicon to stdout

favicon-rover get https://crates.io --out favicon.png # output to favicon.png

favicon-rover get https://crates.io --size 256 # set the size to 256px

favicon-rover get https://crates.io --type webp # set the format to webp

favicon-rover get https://crates.io -o favicons/cratesio -s 50 -t webp # all options

favicon-rover get --help # show help information
```

## Web Server

> [!IMPORTANT]
> You need to enable the `server` feature for this command to be available

Start the web server to expose an API that will fetch favicons

```bash
# Usage: favicon-rover serve [OPTIONS]

favicon-rover serve # start with default options

favicon-rover serve --port 8080 # run on port 8080

favicon-rover serve --host 12.34.56.78 # specify a host

favicon-rover serve --origin https://example.com # only allow requests from example.com

favicon-rover serve -p 1234 --host 0.0.0.0 -o https://example1.com -o /\.example2\.com$/ # all options

favicon-rover serve --help # show help information
```

### API

```h
/{site url}?size={size}
```

`site url` is any valid url to a page that you want the favicon for. Must be URL encoded.

`size` is an integer in pixels to set the returned image. It's optional, and if not included then the best available size will be returned.

Example: `http://localhost:3000/example.com?size=24`

### Fonts

The fallback image generation will attempt to query and load a "sans-serif" font. It will load your system fonts if available as well as any fonts
in the current directory (`pwd`) when favicon-rover is started.

### CORS

By default, any origin is allowed to make a request to this API. To lock it down, use the `--origin` command line options to specify any amount of origins. If an origin starts and ends with `/` it will be treated as a regexp. For example `favicon-rover serve -o http://example1.com -o /\.example2\.com$/` will accept any request from "http://example1.com" or from a subdomain of "example2.com".

> [!TIP]
> We highly recommend setting an origin so your favicon API can't be as easily abused by websites you don't control

## Development

Run `cargo run` to test the binary. You can test the serve command with `cargo run --features server -- serve`.

Run `cargo build` to build in release mode.

## Contributing

If you have any feedback or find a website that favicon rover can't correctly find the favicon for, [create an issue](https://github.com/stevent-team/favicon-rover/issues/new/choose). Contributions are welcome.

## License

Created by Stevent (2023) and licensed under MIT
