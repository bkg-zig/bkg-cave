use clap::{Args, Command, CommandFactory, Parser, Subcommand};

#[derive(Parser)]
pub struct P2pCli {
    #[command(subcommand)]
    pub command: P2pCommand,
}

#[derive(Subcommand)]
pub enum P2pCommand {
    /// Start the P2P server
    Serve(ServeArgs),
    /// Manage peers
    Peers(PeersArgs),
    /// Wallet operations
    Wallet(WalletArgs),
    /// Job marketplace
    Job(JobArgs),
    /// Network diagnostics
    Network(NetworkArgs),
    /// Run diagnostics
    Doctor,
    /// Model management
    Models(ModelsArgs),
    /// Skill management
    Skill(SkillArgs),
    /// Tool management
    Tool(ToolArgs),
    /// Vector operations
    Vector(VectorArgs),
}

#[derive(Args)]
pub struct ServeArgs {
    /// Enable web dashboard
    #[arg(long)]
    pub web: bool,
    /// Enable Ollama integration
    #[arg(long)]
    pub ollama: bool,
    /// Share inference capabilities
    #[arg(long)]
    pub share_inference: bool,
    /// Enable agent mode
    #[arg(long)]
    pub agent: bool,
}

#[derive(Args)]
pub struct PeersArgs {
    #[command(subcommand)]
    pub command: PeersCommand,
}

#[derive(Subcommand)]
pub enum PeersCommand {
    /// List connected peers
    List,
}

#[derive(Args)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Subcommand)]
pub enum WalletCommand {
    /// Show wallet balance
    Balance,
}

#[derive(Args)]
pub struct JobArgs {
    #[command(subcommand)]
    pub command: JobCommand,
}

#[derive(Subcommand)]
pub enum JobCommand {
    /// Submit a new job
    Submit,
    /// Check job status
    Status { id: String },
    /// List all jobs
    List,
}

#[derive(Args)]
pub struct NetworkArgs {
    #[command(subcommand)]
    pub command: NetworkCommand,
}

#[derive(Subcommand)]
pub enum NetworkCommand {
    /// Show network status
    Status,
}

#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommand,
}

#[derive(Subcommand)]
pub enum ModelsCommand {
    /// List available models
    List,
    /// Pull a model
    Pull { name: String },
}

#[derive(Args)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub command: SkillCommand,
}

#[derive(Subcommand)]
pub enum SkillCommand {
    /// List installed skills
    List,
    /// Install a skill
    Install { name: String },
}

#[derive(Args)]
pub struct ToolArgs {
    #[command(subcommand)]
    pub command: ToolCommand,
}

#[derive(Subcommand)]
pub enum ToolCommand {
    /// List available tools
    List,
    /// Execute a tool
    Exec { name: String, args: Vec<String> },
}

#[derive(Args)]
pub struct VectorArgs {
    #[command(subcommand)]
    pub command: VectorCommand,
}

#[derive(Subcommand)]
pub enum VectorCommand {
    /// Query vector store
    Query { text: String },
    /// Index a file
    Index { path: String },
}

pub fn register_commands(app: Command) -> Command {
    app.subcommand(P2pCli::command())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    #[test]
    fn parse_serve() {
        let cli = P2pCli::try_parse_from(["cave", "serve", "--web", "--agent"]).unwrap();
        match cli.command {
            P2pCommand::Serve(args) => {
                assert!(args.web);
                assert!(!args.ollama);
                assert!(args.agent);
            }
            _ => panic!("expected Serve"),
        }
    }

    #[test]
    fn parse_peers_list() {
        let cli = P2pCli::try_parse_from(["cave", "peers", "list"]).unwrap();
        match cli.command {
            P2pCommand::Peers(args) => match args.command {
                PeersCommand::List => {}
            },
            _ => panic!("expected Peers"),
        }
    }

    #[test]
    fn parse_wallet_balance() {
        let cli = P2pCli::try_parse_from(["cave", "wallet", "balance"]).unwrap();
        match cli.command {
            P2pCommand::Wallet(args) => match args.command {
                WalletCommand::Balance => {}
            },
            _ => panic!("expected Wallet"),
        }
    }

    #[test]
    fn parse_job_status() {
        let cli = P2pCli::try_parse_from(["cave", "job", "status", "job-123"]).unwrap();
        match cli.command {
            P2pCommand::Job(args) => match args.command {
                JobCommand::Status { id } => assert_eq!(id, "job-123"),
                _ => panic!("expected Status"),
            },
            _ => panic!("expected Job"),
        }
    }

    #[test]
    fn parse_doctor() {
        let cli = P2pCli::try_parse_from(["cave", "doctor"]).unwrap();
        match cli.command {
            P2pCommand::Doctor => {}
            _ => panic!("expected Doctor"),
        }
    }

    #[test]
    fn register_commands_adds_subcommand() {
        let app = Command::new("test");
        let app = register_commands(app);
        let app = app
            .try_get_matches_from(["test", "cave-p2p", "doctor"])
            .unwrap();
        assert_eq!(app.subcommand_name(), Some("cave-p2p"));
    }
}
