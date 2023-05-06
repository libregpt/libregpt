# LibreGPT

A GPT front-end built for privacy.

### Why?

It's almost impossible to use services that give access to GPT models without giving away your data.
Some need your phone number and/or your email and the ones that don't require an account can still track you when you visit their website.
This project acts as a proxy between you and whatever provider you choose, without needing you to worry about your privacy.

## Features

- Requests are made server-side, you never directly communicate with the providers
- No ads, no trackers

## Instances

None (for now).

## Hosting

If you don't trust public instances you can deploy your own.

### Cargo

```shell
cargo install --git https://github.com/libregpt/libregpt
libregpt
```

## Contributing

Contributions are always welcome!

## License

Licensed under [GNU AGPLv3](LICENSE).
