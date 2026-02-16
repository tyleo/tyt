use clap::Subcommand;
use tl_fbx::TlFbx;

/// The main command for `tlt`, which ties all my command-line tools together.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tlt {
    Fbx {
        #[clap(subcommand)]
        fbx: TlFbx,
    },
}

impl Tlt {
    pub fn execute(self) {
        match self {
            Tlt::Fbx { fbx } => fbx.execute(),
        }
    }
}
