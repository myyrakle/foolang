use serde::Deserialize;

use clap::Args;

use crate::platforms::target::Target;

#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct ConfigOption {
    #[clap(name = "filename")]
    pub filename: String,

    #[clap(long)]
    pub target: Target,
}

#[derive(Clone, Debug, Args)]
#[clap(name = "build")]
pub struct Action {
    #[clap(flatten)]
    pub value: ConfigOption,
}
