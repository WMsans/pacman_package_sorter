mod backend;
mod db;
mod error;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};
use models::{Package, SortKey};
use prettytable::{row, Cell, Row, Table};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List and filter installed packages
    List {
        /// Filter by custom tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Filter by repository (core, extra, multilib, aur, etc.)
        #[arg(short, long)]
        repo: Option<String>,
        /// Show only explicitly installed packages
        #[arg(short, long, conflicts_with = "dependency")]
        explicit: bool,
        /// Show only packages installed as dependencies
        #[arg(short, long)]
        dependency: bool,
        /// Sort packages by a key
        #[arg(short, long, default_value = "name")]
        sort_by: SortKey,
    },
    /// Add a custom tag to a package
    Tag {
        package: String,
        tag: String,
    },
    /// Remove a custom tag from a package
    Untag {
        package: String,
        tag: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List {
            tag,
            repo,
            explicit,
            dependency,
            sort_by,
        } => {
            let packages = backend::get_all_packages().await?;
            let filtered_packages =
                backend::filter_packages(packages, tag.clone(), repo.clone(), *explicit, *dependency);

            let mut sorted_packages = filtered_packages;
            backend::sort_packages(&mut sorted_packages, *sort_by);

            print_packages_table(sorted_packages);
        }
        Commands::Tag { package, tag } => {
            let msg = db::add_tag(package, tag)?;
            println!("{}", msg);
        }
        Commands::Untag { package, tag } => {
            let msg = db::remove_tag(package, tag)?;
            println!("{}", msg);
        }
    }

    Ok(())
}

fn print_packages_table(packages: Vec<Package>) {
    let mut table = Table::new();
    table.add_row(row![b => "Package", "Version", "Repo", "Size (MiB)", "Installed", "Popularity", "Tags"]);

    for pkg in packages {
        let popularity = if pkg.repository == models::Repository::AUR {
            format!(
                "{:.2} ({} votes)",
                pkg.popularity.unwrap_or(0.0),
                pkg.num_votes.unwrap_or(0)
            )
        } else {
            "-".to_string()
        };

        table.add_row(Row::new(vec![
            Cell::new(&pkg.name),
            Cell::new(&pkg.version),
            Cell::new(&format!("{:?}", pkg.repository)),
            Cell::new(&format!("{:.2}", pkg.size)).style_spec("r"), // Right-align size
            Cell::new(&pkg.install_date.format("%Y-%m-%d").to_string()),
            Cell::new(&popularity),
            Cell::new(&pkg.tags.join(", ")),
        ]));
    }
    table.printstd();
}