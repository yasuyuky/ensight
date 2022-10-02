use clap::Parser;
use std::fmt;
use structopt::StructOpt;

#[derive(Debug, Parser, Clone)]
#[clap(rename_all = "lower-case")]
pub enum Vcs {
    #[clap(alias = "gh")]
    GitHub,
    #[clap(alias = "bb")]
    BitBucket,
}

impl fmt::Display for Vcs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GitHub => write!(f, "gh"),
            Self::BitBucket => write!(f, "bb"),
        }
    }
}

impl std::str::FromStr for Vcs {
    type Err = std::string::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gh" => Ok(Self::GitHub),
            "GitHub" => Ok(Self::GitHub),
            "bb" => Ok(Self::BitBucket),
            "BitBucket" => Ok(Self::BitBucket),
            _ => Ok(Self::GitHub),
        }
    }
}
