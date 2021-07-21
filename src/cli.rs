use structopt::StructOpt;

/// A program to retrieve a Minecraft account's bearer token based on the new Microsoft authentication scheme
#[derive(StructOpt, Debug)]
#[structopt()]
pub struct Args {
    /// Email address of account to authenticate
    #[structopt(short, long)]
    pub email: String,

    /// Password of account to authenticate
    #[structopt(short, long)]
    pub password: String,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::from_args()
    }
}
