use serde::Deserialize;

use clap::Args;

#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct ConfigOption {
    #[clap(name = "filename")]
    pub filename: String,
}

#[derive(Clone, Debug, Args)]
#[clap(name = "build")]
pub struct Action {
    #[clap(flatten)]
    pub value: ConfigOption,
}
