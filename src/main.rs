extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate structopt;
extern crate env_logger;
extern crate glob;
extern crate num_cpus;
extern crate reqwest;
extern crate tempfile;

pub mod build;
pub mod config;
pub mod entry;
pub mod error;

use std::env;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "llvmenv",
    about = "Manage multiple LLVM/Clang builds",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
enum LLVMEnv {
    #[structopt(name = "init", about = "Initialize llvmenv")]
    Init {},

    #[structopt(name = "builds", about = "List usable build")]
    Builds {},

    #[structopt(name = "entries", about = "List entries to be built")]
    Entries {},
    #[structopt(name = "build-entry", about = "Build LLVM/Clang")]
    BuildEntry {
        name: String,
        #[structopt(short = "u", long = "update")]
        update: Option<bool>,
        #[structopt(short = "j", long = "nproc")]
        nproc: Option<usize>,
    },

    #[structopt(name = "current", about = "Show the name of current build")]
    Current {
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
    #[structopt(name = "prefix", about = "Show the prefix of the current build")]
    Prefix {
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },

    #[structopt(name = "global", about = "Set the build to use (global)")]
    Global { name: String },
    #[structopt(name = "local", about = "Set the build to use (local)")]
    Local {
        name: String,
        #[structopt(short = "p", long = "path", parse(from_os_str))]
        path: Option<PathBuf>,
    },

    #[structopt(name = "zsh", about = "Setup Zsh integration")]
    Zsh {},
}

fn main() -> error::Result<()> {
    env_logger::init();
    let opt = LLVMEnv::from_args();
    match opt {
        LLVMEnv::Init {} => config::init_config()?,

        LLVMEnv::Builds {} => {
            let builds = build::builds()?;
            let max = builds.iter().map(|b| b.name().len()).max().unwrap();
            for b in &builds {
                println!(
                    "{name:<width$}: {prefix}",
                    name = b.name(),
                    prefix = b.prefix().display(),
                    width = max
                );
            }
        }

        LLVMEnv::Entries {} => {
            let entries = config::load_entries()?;
            for entry in &entries {
                println!("{}", entry.get_name());
            }
        }
        LLVMEnv::BuildEntry {
            name,
            update,
            nproc,
        } => {
            let entry = config::load_entry(&name)?;
            let update = update.unwrap_or(false);
            let nproc = nproc.unwrap_or(num_cpus::get());
            entry.checkout().unwrap();
            if update {
                entry.fetch().unwrap();
            }
            entry.build(nproc).unwrap();
        }

        LLVMEnv::Current { verbose } => {
            let build = build::seek_build()?;
            println!("{}", build.name());
            if verbose {
                if let Some(env) = build.env_path() {
                    eprintln!("set by {}", env.display());
                }
            }
        }
        LLVMEnv::Prefix { verbose } => {
            let build = build::seek_build()?;
            println!("{}", build.prefix().display());
            if verbose {
                if let Some(env) = build.env_path() {
                    eprintln!("set by {}", env.display());
                }
            }
        }

        LLVMEnv::Global { name } => {
            let build = build::Build::from_name(&name);
            if build.exists() {
                build.set_global()?;
            } else {
                eprintln!("Build '{}' does not exists", name);
                exit(1);
            }
        }
        LLVMEnv::Local { name, path } => {
            let build = build::Build::from_name(&name);
            let path = path.unwrap_or(env::current_dir()?);
            if build.exists() {
                build.set_local(&path)?;
            } else {
                eprintln!("Build '{}' does not exists", name);
                exit(1);
            }
        }

        LLVMEnv::Zsh {} => {
            let src = include_str!("../llvmenv.zsh");
            println!("{}", src);
        }
    }
    Ok(())
}
