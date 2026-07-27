use argh::FromArgs;
use xshell::Shell;

use crate::args::Args;
use crate::prepare::{Prepare, PreparedCommand};

use super::lints::Lints;
use super::msrv::Msrv;
use super::test_features::TestFeatures;

#[derive(FromArgs, Default)]
#[argh(subcommand, name = "ci")]
/// Run the full CI suite (requires msrv, nightly, wasm-pack, wasmtime)
pub struct Ci {}

impl Prepare for Ci {
    fn prepare<'a>(&self, sh: &'a Shell, args: &Args) -> Vec<PreparedCommand<'a>> {
        let mut cmds = Vec::new();
        cmds.extend(Lints {}.prepare(sh, args));
        cmds.extend(TestFeatures::default().prepare(sh, args));
        cmds.extend(Msrv::default().prepare(sh, args));
        cmds
    }
}
