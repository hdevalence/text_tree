use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, StructOpt)]
#[structopt(name = "dump", about = "Dump parsed HTML or CSS as an AST.")]
struct Opts {
    #[structopt(long = "log", env = "RUST_LOG")]
    log: Option<EnvFilter>,

    #[structopt(subcommand)]
    kind: Kind,
}

#[derive(Debug, StructOpt)]
enum Kind {
    Html {
        #[structopt(name = "file", parse(from_os_str))]
        file: PathBuf,
    },
    Css {
        #[structopt(name = "FILE", parse(from_os_str))]
        file: PathBuf,
    },
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();
    if let Some(log) = opts.log {
        tracing_subscriber::fmt().with_env_filter(log).init();
    }
    match opts.kind {
        Kind::Html { file } => {
            let s = std::fs::read_to_string(file)?;
            match s.parse::<text_tree::content_tree::Node>() {
                Ok(node) => println!("{}", node),
                Err(e) => eprintln!("parse error: {}", e),
            }
        }
        Kind::Css { file: _ } => todo!("eliza needs to implement this!"),
    }
    Ok(())
}
