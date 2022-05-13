use powerpack::{Icon, Item};
use std::{env, fmt::Display, iter::successors, num::ParseIntError, str::FromStr};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
enum Error {
    InvalidHint(String),
    InvalidBase(String, ParseIntError),
    BaseOutOfRange(u8),
    InvalidNumber(String, Format, ParseIntError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidHint(hint) => {
                write!(f, "Invalid base hint: {hint}")
            }
            Error::InvalidBase(base, e) => {
                write!(f, "Base hint '{base}' could not be parsed as uint8: {e}")
            }
            Error::BaseOutOfRange(base) => {
                write!(f, "Base {base} is not in [2, 36]")
            }
            Error::InvalidNumber(number, format, e) => {
                write!(f, "Number '{number}' could not be parsed as {format}: {e}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidBase(_, e) => Some(e),
            Error::InvalidNumber(_, _, e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Format {
    Binary,
    Octal,
    Decimal,
    Hex,
    Custom(u8),
}

impl Format {
    fn parse(self, value: &str) -> Result<Number> {
        let base = match self {
            Format::Binary => 2,
            Format::Octal => 8,
            Format::Decimal => 10,
            Format::Hex => 16,
            Format::Custom(base) => base,
        };

        let value = i64::from_str_radix(value, u32::from(base))
            .map_err(|e| Error::InvalidNumber(value.to_string(), self, e))?;
        Ok(Number {
            value,
            format: self,
        })
    }

    fn format(self, value: i64) -> String {
        match self {
            Format::Binary => format!("{:b}", value),
            Format::Octal => format!("{:o}", value),
            Format::Decimal => format!("{}", value),
            Format::Hex => format!("{:x}", value),
            Format::Custom(_) => unimplemented!("Custom base render is not supported"),
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Format::Binary => "icons/Binary.png",
            Format::Octal => "icons/Octal.png",
            Format::Decimal => "icons/Decimal.png",
            Format::Hex => "icons/Hexadecimal.png",
            Format::Custom(_) => "icons/Custom.png",
        }
    }

    fn item(self, value: i64) -> Item {
        let item = self.format(value);
        Item::new(&item)
            .subtitle(self.to_string())
            .valid(true)
            .arg(item)
            .icon(Icon::with_image(self.icon()))
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Binary => f.write_str("Binary"),
            Format::Octal => f.write_str("Octal"),
            Format::Decimal => f.write_str("Decimal"),
            Format::Hex => f.write_str("Hexadecimal"),
            Format::Custom(base) => write!(f, "Base{base}"),
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        fn prefix_of(s: &str) -> Option<&str> {
            Some(&s[..s.len().checked_sub(1)?])
        }

        let hint = s.to_lowercase();
        let hint = hint.as_str();
        if "binary".starts_with(hint) || hint == "0b" {
            Ok(Self::Binary)
        } else if "octal".starts_with(hint) || hint == "0o" {
            Ok(Self::Octal)
        } else if "decimal".starts_with(hint) || matches!(hint, "0d" | "0") {
            Ok(Self::Decimal)
        } else if "hexadecimal".starts_with(hint) || matches!(hint, "0x" | "x") {
            Ok(Self::Hex)
        } else if let Some(base) = successors(Some("base"), |base| prefix_of(base))
            .find_map(|base| hint.strip_prefix(base))
        {
            let base = base
                .parse()
                .map_err(|e| Error::InvalidBase(base.to_string(), e))?;

            (2..=36)
                .contains(&base)
                .then(|| Self::Custom(base))
                .ok_or(Error::BaseOutOfRange(base))
        } else {
            Err(Error::InvalidHint(s.to_string()))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Number {
    value: i64,
    format: Format,
}

impl Number {
    fn parse(value: &str, hint: Option<&str>) -> Result<Self> {
        if let Some(format) = hint.map(Format::from_str).transpose()? {
            format.parse(value)
        } else if let Some(value) = value.strip_prefix("0b") {
            Format::Binary.parse(value)
        } else if let Some(value) = value.strip_prefix("0o") {
            Format::Octal.parse(value)
        } else if let Some(value) = value.strip_prefix("0d") {
            Format::Decimal.parse(value)
        } else if let Some(value) = value.strip_prefix("0x") {
            Format::Hex.parse(value)
        } else {
            Format::Decimal
                .parse(value)
                .or_else(|_| Format::Hex.parse(value))
        }
    }

    fn convert(self) -> Vec<Item> {
        [Format::Binary, Format::Octal, Format::Decimal, Format::Hex]
            .into_iter()
            .filter_map(|f| (f != self.format).then(|| f.item(self.value)))
            .collect()
    }
}

fn run() -> Vec<Item> {
    // Alfred passes in a single argument for the user query.
    match env::args().nth(1) {
        Some(query) => {
            let mut parts = query.split_whitespace();
            let number = parts.next().unwrap();
            let hint = parts.next();
            let number = Number::parse(number, hint);
            match number {
                Ok(number) => number.convert(),
                Err(e) => vec![Item::new(e.to_string()).valid(false)],
            }
        }
        None => vec![Item::new("Provide a number with an optional base").valid(false)],
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let items = run();
    powerpack::output(items)?;
    Ok(())
}
