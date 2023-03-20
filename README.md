# Monsoon

Monsoon is a library for accessing weather data produced by [The Norwegian Meteorological Institute](https://www.met.no/en). Most notably, this data is used on [Yr.no](https://www.yr.no/en).

- [Documentation](https://docs.rs/monsoon)
- [Crates.io](https://crates.io/crates/monsoon)

## Examples

```rust
let monsoon = Monsoon::new("test.com support@test.com")?;

// Prague
let response = monsoon.get(50.0880, 14.4207).await?;
let body = response.body()?;
```

<img width="1788" alt="image" src="https://user-images.githubusercontent.com/20820/226291249-1a6d5e49-ab31-4928-bbef-0e3acba98292.png">

See [example](https://github.com/jiripospisil/monsoon/tree/master/example) for more details.

## License

- MIT license
