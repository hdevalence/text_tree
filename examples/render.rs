use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, StructOpt)]
#[structopt(name = "render", about = "render parsed html styled with parsed tascading style sheets")]
struct Opts {
    #[structopt(long = "log", env = "RUST_LOG")]
    log: Option<EnvFilter>,

    #[structopt(name = "html", parse(from_os_str))]
    html_file: PathBuf,

    #[structopt(name = "tss", parse(from_os_str))]
    tss_file: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use text_tree::layout::{Dimensions, Rect, LayoutBox};

    let opts = Opts::from_args();
    if let Some(log) = opts.log {
        tracing_subscriber::fmt().with_env_filter(log).init();
    }

    let s = std::fs::read_to_string(opts.html_file)?;
    let html = s.parse::<text_tree::content_tree::Node>()?;

    let s = std::fs::read_to_string(opts.tss_file)?;
    let stylesheet = s.parse::<text_tree::style::Stylesheet>()?;

    let styled_root = text_tree::style_tree::style_tree(&html, &stylesheet);

    tracing::debug!("styled root: {:#?}", styled_root);

    let mut layout_root = text_tree::layout::build_layout_tree(&styled_root);

    tracing::debug!("layout root: {:#?}", layout_root);

    layout_root.layout(&Dimensions::default());

    // tracing::debug!("dims: {:#?}", layout_root.dimensions());
    text_tree::print_boxes(&layout_root);

    let mut c = text_tree::display::DebugCanvas::new(80, 35);

    c.paint(&text_tree::display::build_display_list(&layout_root));

    c.print();

    Ok(())
}
