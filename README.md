# LibreGPT

A GPT front-end built for privacy.

> **Note**
> This project is under heavy development.

### Why?

It's almost impossible to use services that give access to GPT models without giving away your data.
Some need your phone number and/or your email and the ones that don't require an account can still track you when you visit their website.
This project acts as a proxy between you and whatever provider you choose, without needing you to worry about your privacy.

## Features

- Requests are made server-side, you never directly communicate with the providers
- No ads, no trackers

## Instances

None (for now).

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
