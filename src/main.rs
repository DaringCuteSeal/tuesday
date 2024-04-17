use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use graph::{ErrorType, TaskState};

mod graph;
mod save;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a node to the graph
    Add {
        /// Adds a root node
        #[arg(short, long)]
        root: bool,

        /// Parent of node
        #[arg(short, long)]
        parent: Option<String>,

        /// What message should the node have
        #[arg(short, long)]
        message: String,
    },

    /// Removes a node from the graph
    Rm {
        /// Node index to remove
        #[arg(short, long)]
        target: String,

        /// Whether to remove child nodes recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Links a node to a target node
    Link {
        /// Node to link from
        #[arg(short, long)]
        from: String,

        /// Node to link to
        #[arg(short, long)]
        to: String,
    },

    /// Unlinks a node
    Unlink {
        #[arg(short, long)]
        from: String,

        #[arg(short, long)]
        to: String,
    },

    /// Marks a node as completed
    Check {
        #[arg(short, long)]
        target: String,
    },

    /// Marks a node as incomplete
    Uncheck {
        #[arg(short, long)]
        target: String,
    },

    /// Adds an alias for a node
    Alias {
        #[arg(short, long)]
        target: String,

        #[arg(short, long)]
        alias: String,
    },

    Unalias {
        #[arg(short, long)]
        target: String,
    },

    /// Lists children nodes
    List {
        #[arg(short, long)]
        target: Option<String>,

        /// No value or 0 = infinite depth
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Displays statistics of a node
    Stats {
        #[arg(short, long)]
        target: String,
    },

    /// Compresses and cleans up the graph
    Clean,
}

fn handle_command(commands: Commands, graph: &mut graph::TaskGraph) -> Result<()> {
    match commands {
        Commands::Add {
            root,
            parent,
            message,
        } => {
            if root {
                graph.insert_root(message);
            } else if let Some(target) = parent {
                graph.insert_child(message, target)?;
            } else {
                bail!("Did not specify whether to add as root or as a child node!");
            }
            Ok(())
        }
        Commands::Rm { target, recursive } => {
            if recursive {
                graph.remove_children_recursive(target)?;
            } else {
                graph.remove(target)?;
            }
            Ok(())
        }
        Commands::Link { from, to } => {
            graph.link(from, to)?;
            Ok(())
        }
        Commands::Unlink { from, to } => {
            graph.unlink(from, to)?;
            Ok(())
        }
        Commands::Check { target: index } => {
            graph.set_state(index, TaskState::Complete)?;
            Ok(())
        }
        Commands::Uncheck { target: index } => {
            graph.set_state(index, TaskState::None)?;
            Ok(())
        }
        Commands::Alias { target, alias } => {
            graph.set_alias(target, alias)?;
            Ok(())
        }
        Commands::Unalias { target } => {
            graph.unset_alias(target)?;
            Ok(())
        }
        Commands::List { target, depth } => {
            match target {
                None => graph.list_roots()?,
                Some(t) => graph.list_children(t, depth.unwrap_or(0))?,
            }
            Ok(())
        }
        Commands::Stats { target } => {
            graph.display_stats(target)?;
            Ok(())
        }
        Commands::Clean => {
            *graph = graph.clean()?;
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let mut graph = save::load_global()?;
    let cli = Cli::parse();

    if cli.command.is_none() {
        println!("Welcome to Tuesday");
        save::save_global(&save::Config::new(&graph))?;
        return Ok(());
    }

    let result: Result<()> = handle_command(cli.command.unwrap(), &mut graph);
    if let Err(e) = result {
        match e.downcast::<ErrorType>()? {
            ErrorType::InvalidIndex(index) => println!("Invalid index: {index}"),
            ErrorType::InvalidAlias(alias) => println!("Invalid alias: {alias}"),
            ErrorType::GraphLooped(index) => println!("Graph looped at index: {index}"),
        }
    }

    save::save_global(&save::Config::new(&graph))?;

    Ok(())
}
