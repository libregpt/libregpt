<div align="center">
  <h1>LibreGPT</h1>
  <p>A GPT front-end built for privacy.</p>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/libregpt/assets/main/screenshot-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/libregpt/assets/main/screenshot-light.png">
    <img alt="" src="https://raw.githubusercontent.com/libregpt/assets/main/screenshot-light.png">
  </picture>
</div>

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#motivation">Motivation</a></li>
    <li><a href="#features">Features</a></li>
    <li><a href="#instances">Instances</a></li>
    <li><a href="#hosting">Hosting</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>

## Motivation

It's almost impossible to use services that give access to GPT models without giving away your data.
Some need your phone number and/or your email and the ones that don't require an account can still track you when you visit their website.
This project acts as a proxy between you and whatever provider you choose, without needing you to worry about your privacy.

## Features

- Requests are made server-side, you never directly communicate with the providers
- No ads, no trackers

## Instances

| URL                     | Network  | Location | Cloudflare                       |
|-------------------------|----------|----------|----------------------------------|
| https://chat.samue.land | Clearnet | 🇺🇸 US  | $\textcolor{green}{\textsf{No}}$ |

If you don't trust public instances and want to deploy your own take a look at the [hosting](#hosting) section.

## Hosting

First, you need to clone the repository and cd into it.

```shell
git clone https://github.com/libregpt/libregpt
cd libregpt
```

Then you can choose between:

### [Docker](https://www.docker.com) (recommended)

```shell
docker build -t libregpt .
docker run -d --name libregpt -p 80:80 libregpt
```

### Native

You need to install [Cargo](https://doc.rust-lang.org/stable/cargo/) and [Trunk](https://trunkrs.dev).

If you have [Make](https://www.gnu.org/software/make/) you can run

```shell
make run MODE=release
```

otherwise

```shell
trunk build --release --public-url /pkg index.html
cargo run --release --features=ssr
```

## Contributing

Contributions are always welcome!

## License

Licensed under [GNU AGPLv3](LICENSE).
