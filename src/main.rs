mod args;
mod output;

fn main() {
    if let Err(err) = real_main() {
        output::print_error(&err);
        std::process::exit(1);
    }
}

fn real_main() -> anyhow::Result<()> {
    use clap::Parser as _;

    let cli = args::Cli::parse();

    match cli.cmd {
        args::Command::Set { image } => nayu_infra::ipc::client::set(image),
        args::Command::Daemon => nayu_infra::ipc::server::run_daemon(),
        args::Command::Status => nayu_infra::ipc::client::status(),
    }
}
