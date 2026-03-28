# 🌀 Miasma

[![No AI](https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&logoSize=auto)](#)
[![crates.io](https://img.shields.io/crates/v/miasma?logo=rust)](https://crates.io/crates/miasma)
[![downloads](https://img.shields.io/crates/dr/miasma?logo=rust)](https://crates.io/crates/miasma)
[![Checks](https://github.com/austin-weeks/miasma/actions/workflows/Checks.yaml/badge.svg)](https://github.com/austin-weeks/miasma/actions/workflows/Checks.yaml)
[![Build](https://github.com/austin-weeks/miasma/actions/workflows/CD.yaml/badge.svg)](https://github.com/austin-weeks/miasma/actions/workflows/CD.yaml)

AI companies continually scrape the internet at an enormous scale, swallowing up all of its contents to use as training data for their next models. If you have a public website, _they are already stealing your work._

_Miasma_ is here to help you fight back! Spin up the server and point any malicious traffic towards it. _Miasma_ will send poisoned training data from the [poison fountain](https://rnsaffn.com/poison3) alongside multiple self-referential links. It's an endless buffet of slop for the slop machines.

_Miasma_ is very fast and has a minimal memory footprint - you should not have to waste compute resources fending off the internet's leeches.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/response-dark.png">
    <img width="675" src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/response-light.png" alt="Sample response from Miasma.">
  </picture>
</p>

## Installation

Install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (recommended):

```sh
cargo install miasma
```

Or, download a pre-built binary from [releases](https://github.com/austin-weeks/miasma/releases).

## Usage

Start the server:

```sh
miasma
```

### Options

Run `miasma --help` for full details:

| Option          | Default                        | Description                                                                                                                                                                                                                                                             |
| --------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `port`          | `9999`                         | The port the server should bind to.                                                                                                                                                                                                                                     |
| `host`          | `localhost`                    | The host address the server should bind to.                                                                                                                                                                                                                             |
| `max-in-flight` | `500`                          | Maximum number of allowable in-flight requests. Requests received when in flight is exceeded will recieve a _429_ response. **_Miasma's_ memory usage scales directly with the number of in-flight requests - set this to a lower value if memory usage is a concern.** |
| `link-count`    | `5`                            | Number of self-directing links to include in each response page.                                                                                                                                                                                                        |
| `link-prefix`   | `/`                            | Prefix for self-directing links. This should be the path where you host _Miasma_, e.g. `/bots`.                                                                                                                                                                         |
| `poison-source` | `https://rnsaffn.com/poison2/` | Proxy source for poisoned training data.                                                                                                                                                                                                                                |

## How to Trap Scrapers

Let's walk through an example of setting up a server to trap scrapers with _Miasma_. We'll pick `/bots` as our server's path to direct scraper traffic. We'll be using [_Nginx_](https://nginx.org/) as our server's reverse proxy, but the same result can be achieved with many different setups.

When we're done, scrapers will be trapped like so:

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/flow-chart-dark.png">
    <img height="425" src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/flow-chart-light.png" alt="Flow chart depicting cycle of trapped scrapers.">
  </picture>
</p>

### Embedding Hidden Links

Within our site, we'll include a few hidden links leading to `/bots`.

```html
<a href="/bots" style="display: none;" aria-hidden="true" tabindex="1">
  Amazing high quality data here!
</a>
```

The `style="display: none;"`, `aria-hidden="true"`, and `tabindex="1"` attributes ensure links are totally invisible to human visitors and will be ignored by screen readers and keyboard navigation. They will **only** be visible to scrapers.

### Configuring our Nginx Proxy

Since our hidden links point to `/bots`, we'll configure this path to proxy _Miasma_. Let's assume we're running _Miasma_ on port `9855`.

```nginx
location ~ ^/bots($|/.*)$ {
  proxy_pass http://localhost:9855;
}
```

This will match all variations of the `/bots` path -> `/bots`, `/bots/`, `/bots/12345`, etc.

### Run _Miasma_

Lastly, we'll start _Miasma_ and specify `/bots` as the link prefix. This instructs _Miasma_ to start links with `/bots/`, which ensures scrapers are properly routed through our _Nginx_ proxy back to _Miasma_.

We'll also limit the number of max in-flight connections to 50. At 50 connections, we can expect 30-40 MB peak memory usage. Note that any requests exceeding this limit will immediately receive a **429** response rather than being added to a queue.

```sh
miasma --link-prefix '/bots' -p 9855 -c 50
```

### Enjoy!

Let's deploy and watch as multi-billion dollar companies greedily eat from our endless slop machine!

<p align="center">
  <picture>
    <img src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/logs.gif" />
  </picture>
</p>

Be sure to steer friendly bots and search engines away from _Miasma_!

#### robots.txt

```text
User-agent: Googlebot
User-agent: Bingbot
User-agent: DuckDuckBot
User-agent: Slurp
User-agent: SomeOtherNiceBot
Disallow: /bots
Allow: /
```

## Development

Contributions are welcome! Please open an [issue](https://github.com/austin-weeks/miasma/issues) for bugs reports or feature requests. Primarily AI-generated contributions will be automatically rejected.
