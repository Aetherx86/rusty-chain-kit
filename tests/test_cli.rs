use clap::Parser;
use rusty_chain_kit::cli::{
    BuildArgs, ChainArgs, ChainSubcommand, Cli, Commands, CompileArgs, DecompileArgs,
    DevnetArgs, DevnetSubcommand, ExtractArgs, NewArgs, NewKind, NodeArgs, NodeSubcommand,
    ReposArgs, ReposSubcommand, RpcArgs, RpcSubcommand, ToolsArgs, ToolsSubcommand, UpdateArgs,
};
use rusty_chain_kit::workspace::Strategy;

#[test]
fn test_cli_empty_defaults() {
    let cli = Cli::try_parse_from(["para"]).unwrap();
    assert!(cli.command.is_none());
    assert!(cli.passthrough.is_empty());
}

#[test]
fn test_cli_repos_list() {
    let cli = Cli::try_parse_from(["para", "repos", "list"]).unwrap();
    match cli.command.unwrap() {
        Commands::Repos(ReposArgs { action: Some(ReposSubcommand::List) }) => {}
        _ => panic!("Expected repos list"),
    }
}

#[test]
fn test_cli_repos_status() {
    let cli = Cli::try_parse_from(["para", "repos", "status"]).unwrap();
    match cli.command.unwrap() {
        Commands::Repos(ReposArgs { action: Some(ReposSubcommand::Status) }) => {}
        _ => panic!("Expected repos status"),
    }
}

#[test]
fn test_cli_repos_default_subcommand() {
    let cli = Cli::try_parse_from(["para", "repos"]).unwrap();
    match cli.command.unwrap() {
        Commands::Repos(ReposArgs { action: None }) => {}
        _ => panic!("Expected repos with no action"),
    }
}

#[test]
fn test_cli_repos_sync_options() {
    let cli = Cli::try_parse_from([
        "para", "repos", "sync", "--strategy", "merge", "--no-stash", "--dry-run", "foundry", "solar"
    ]).unwrap();
    match cli.command.unwrap() {
        Commands::Repos(ReposArgs {
            action: Some(ReposSubcommand::Sync { repos, strategy, no_stash, dry_run, push }),
        }) => {
            assert_eq!(repos, vec!["foundry", "solar"]);
            assert_eq!(strategy, Some(Strategy::Merge));
            assert!(no_stash);
            assert!(dry_run);
            assert!(!push);
        }
        _ => panic!("Expected repos sync"),
    }
}

#[test]
fn test_cli_repos_fetch() {
    let cli = Cli::try_parse_from(["para", "repos", "fetch", "reth"]).unwrap();
    match cli.command.unwrap() {
        Commands::Repos(ReposArgs {
            action: Some(ReposSubcommand::Fetch { repos }),
        }) => {
            assert_eq!(repos, vec!["reth"]);
        }
        _ => panic!("Expected repos fetch"),
    }
}

#[test]
fn test_cli_update_flags() {
    let cli = Cli::try_parse_from([
        "para", "update", "--check", "--rebuild", "--skip-build", "--strategy", "rebase"
    ]).unwrap();
    match cli.command.unwrap() {
        Commands::Update(UpdateArgs { check, rebuild, skip_build, strategy }) => {
            assert!(check);
            assert!(rebuild);
            assert!(skip_build);
            assert_eq!(strategy, Some(Strategy::Rebase));
        }
        _ => panic!("Expected update command"),
    }
}

#[test]
fn test_cli_update_short_flags() {
    let cli = Cli::try_parse_from(["para", "update", "-c", "-r", "-s"]).unwrap();
    match cli.command.unwrap() {
        Commands::Update(UpdateArgs { check, rebuild, skip_build, strategy }) => {
            assert!(check);
            assert!(rebuild);
            assert!(skip_build);
            assert!(strategy.is_none());
        }
        _ => panic!("Expected update command with short flags"),
    }
}

#[test]
fn test_cli_build_args() {
    let cli = Cli::try_parse_from(["para", "build", "foundry", "-j", "8", "-p", "release", "--stale"]).unwrap();
    match cli.command.unwrap() {
        Commands::Build(BuildArgs { repo, jobs, profile, stale }) => {
            assert_eq!(repo, Some("foundry".into()));
            assert_eq!(jobs, Some(8));
            assert_eq!(profile, Some("release".into()));
            assert!(stale);
        }
        _ => panic!("Expected build command"),
    }
}

#[test]
fn test_cli_tools_list() {
    let cli = Cli::try_parse_from(["para", "tools", "list"]).unwrap();
    match cli.command.unwrap() {
        Commands::Tools(ToolsArgs { action: Some(ToolsSubcommand::List) }) => {}
        _ => panic!("Expected tools list"),
    }
}

#[test]
fn test_cli_tools_run() {
    let cli = Cli::try_parse_from(["para", "tools", "run", "solar", "--emit", "abi"]).unwrap();
    match cli.command.unwrap() {
        Commands::Tools(ToolsArgs { action: Some(ToolsSubcommand::Run { bin, args }) }) => {
            assert_eq!(bin, "solar");
            assert_eq!(args, vec!["--emit", "abi"]);
        }
        _ => panic!("Expected tools run"),
    }
}

#[test]
fn test_cli_rpc_list() {
    let cli = Cli::try_parse_from(["para", "rpc", "list"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::List) }) => {}
        _ => panic!("Expected rpc list"),
    }
}

#[test]
fn test_cli_rpc_default() {
    let cli = Cli::try_parse_from(["para", "rpc", "default"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::Default { name: None }) }) => {}
        _ => panic!("Expected rpc default getter"),
    }
}

#[test]
fn test_cli_rpc_default_set() {
    let cli = Cli::try_parse_from(["para", "rpc", "default", "sepolia"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::Default { name: Some(name) }) }) => {
            assert_eq!(name, "sepolia");
        }
        _ => panic!("Expected rpc default setter"),
    }
}

#[test]
fn test_cli_rpc_resolve() {
    let cli = Cli::try_parse_from(["para", "rpc", "resolve", "arbitrum"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::Resolve { query }) }) => {
            assert_eq!(query, "arbitrum");
        }
        _ => panic!("Expected rpc resolve"),
    }
}

#[test]
fn test_cli_rpc_add() {
    let cli = Cli::try_parse_from(["para", "rpc", "add", "myrpc", "https://myrpc.io"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::Add { name, url }) }) => {
            assert_eq!(name, "myrpc");
            assert_eq!(url, "https://myrpc.io");
        }
        _ => panic!("Expected rpc add"),
    }
}

#[test]
fn test_cli_rpc_remove() {
    let cli = Cli::try_parse_from(["para", "rpc", "remove", "myrpc"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::Remove { name }) }) => {
            assert_eq!(name, "myrpc");
        }
        _ => panic!("Expected rpc remove"),
    }
}

#[test]
fn test_cli_devnet_up_defaults() {
    let cli = Cli::try_parse_from(["para", "devnet", "up"]).unwrap();
    match cli.command.unwrap() {
        Commands::Devnet(DevnetArgs {
            action: Some(DevnetSubcommand::Up { port, chain_id, block_time, fork, fork_block, persist }),
        }) => {
            assert_eq!(port, 8545);
            assert!(chain_id.is_none());
            assert_eq!(block_time, 0);
            assert!(fork.is_none());
            assert!(fork_block.is_none());
            assert!(!persist);
        }
        _ => panic!("Expected devnet up defaults"),
    }
}

#[test]
fn test_cli_devnet_up_custom() {
    let cli = Cli::try_parse_from([
        "para", "devnet", "up", "-p", "8555", "--chain-id", "1337", "--block-time", "2", "--fork", "https://eth.llamarpc.com", "--persist"
    ]).unwrap();
    match cli.command.unwrap() {
        Commands::Devnet(DevnetArgs {
            action: Some(DevnetSubcommand::Up { port, chain_id, block_time, fork, fork_block, persist }),
        }) => {
            assert_eq!(port, 8555);
            assert_eq!(chain_id, Some(1337));
            assert_eq!(block_time, 2);
            assert_eq!(fork, Some("https://eth.llamarpc.com".into()));
            assert!(fork_block.is_none());
            assert!(persist);
        }
        _ => panic!("Expected devnet up custom"),
    }
}

#[test]
fn test_cli_devnet_subcommands() {
    assert!(matches!(
        Cli::try_parse_from(["para", "devnet", "down"]).unwrap().command.unwrap(),
        Commands::Devnet(DevnetArgs { action: Some(DevnetSubcommand::Down) })
    ));
    assert!(matches!(
        Cli::try_parse_from(["para", "devnet", "status"]).unwrap().command.unwrap(),
        Commands::Devnet(DevnetArgs { action: Some(DevnetSubcommand::Status) })
    ));
    assert!(matches!(
        Cli::try_parse_from(["para", "devnet", "accounts"]).unwrap().command.unwrap(),
        Commands::Devnet(DevnetArgs { action: Some(DevnetSubcommand::Accounts) })
    ));
    assert!(matches!(
        Cli::try_parse_from(["para", "devnet", "logs", "-n", "100"]).unwrap().command.unwrap(),
        Commands::Devnet(DevnetArgs { action: Some(DevnetSubcommand::Logs { lines: 100 }) })
    ));
}

#[test]
fn test_cli_node_subcommands() {
    assert!(matches!(
        Cli::try_parse_from(["para", "node", "status"]).unwrap().command.unwrap(),
        Commands::Node(NodeArgs { action: Some(NodeSubcommand::Status) })
    ));
    assert!(matches!(
        Cli::try_parse_from(["para", "node", "down"]).unwrap().command.unwrap(),
        Commands::Node(NodeArgs { action: Some(NodeSubcommand::Down) })
    ));
    let up = Cli::try_parse_from(["para", "node", "up", "-p", "9545", "--fresh"]).unwrap();
    match up.command.unwrap() {
        Commands::Node(NodeArgs { action: Some(NodeSubcommand::Up { port, fresh, .. }) }) => {
            assert_eq!(port, 9545);
            assert!(fresh);
        }
        _ => panic!("Expected node up"),
    }
}

#[test]
fn test_cli_chain_info() {
    let cli = Cli::try_parse_from(["para", "chain", "info", "--rpc", "https://rpc.ankr.com/eth"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { rpc, action: ChainSubcommand::Info }) => {
            assert_eq!(rpc, Some("https://rpc.ankr.com/eth".into()));
        }
        _ => panic!("Expected chain info"),
    }
}

#[test]
fn test_cli_chain_block() {
    let cli = Cli::try_parse_from(["para", "chain", "block", "latest"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Block { id }, .. }) => {
            assert_eq!(id, Some("latest".into()));
        }
        _ => panic!("Expected chain block latest"),
    }
}

#[test]
fn test_cli_chain_tx() {
    let cli = Cli::try_parse_from(["para", "chain", "tx", "0x1234567890abcdef"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Tx { hash }, .. }) => {
            assert_eq!(hash, "0x1234567890abcdef");
        }
        _ => panic!("Expected chain tx"),
    }
}

#[test]
fn test_cli_chain_address_and_code() {
    let cli = Cli::try_parse_from(["para", "chain", "address", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Address { address }, .. }) => {
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        }
        _ => panic!("Expected chain address"),
    }

    let cli_code = Cli::try_parse_from(["para", "chain", "code", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"]).unwrap();
    match cli_code.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Code { address }, .. }) => {
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        }
        _ => panic!("Expected chain code"),
    }
}

#[test]
fn test_cli_compile() {
    let cli = Cli::try_parse_from(["para", "compile", "contracts/Token.sol", "--forge"]).unwrap();
    match cli.command.unwrap() {
        Commands::Compile(CompileArgs { path, forge }) => {
            assert_eq!(path.to_str().unwrap(), "contracts/Token.sol");
            assert!(forge);
        }
        _ => panic!("Expected compile"),
    }
}

#[test]
fn test_cli_audit() {
    let cli = Cli::try_parse_from(["para", "audit", "contracts/"]).unwrap();
    match cli.command.unwrap() {
        Commands::Audit(rusty_chain_kit::cli::AuditArgs { path }) => {
            assert_eq!(path.to_str().unwrap(), "contracts/");
        }
        _ => panic!("Expected audit"),
    }
}

#[test]
fn test_cli_decompile() {
    let cli = Cli::try_parse_from(["para", "decompile", "0xContractAddr", "--rpc", "https://eth.llamarpc.com"]).unwrap();
    match cli.command.unwrap() {
        Commands::Decompile(DecompileArgs { target, rpc }) => {
            assert_eq!(target, "0xContractAddr");
            assert_eq!(rpc, Some("https://eth.llamarpc.com".into()));
        }
        _ => panic!("Expected decompile"),
    }
}

#[test]
fn test_cli_extract() {
    let cli = Cli::try_parse_from([
        "para", "extract", "logs", "-b", "19000000:19001000", "-o", "./data", "-f", "parquet", "--rpc", "mainnet"
    ]).unwrap();
    match cli.command.unwrap() {
        Commands::Extract(ExtractArgs { dataset, blocks, out, format, rpc }) => {
            assert_eq!(dataset, "logs");
            assert_eq!(blocks, Some("19000000:19001000".into()));
            assert_eq!(out, Some("./data".into()));
            assert_eq!(format, Some("parquet".into()));
            assert_eq!(rpc, Some("mainnet".into()));
        }
        _ => panic!("Expected extract"),
    }
}

#[test]
fn test_cli_new_templates() {
    let cli_bot = Cli::try_parse_from(["para", "new", "bot", "mev-searcher", "./bots"]).unwrap();
    match cli_bot.command.unwrap() {
        Commands::New(NewArgs { kind: NewKind::Bot { name, out } }) => {
            assert_eq!(name, "mev-searcher");
            assert_eq!(out, Some("./bots".into()));
        }
        _ => panic!("Expected new bot"),
    }

    let cli_script = Cli::try_parse_from(["para", "new", "script", "deploy-token"]).unwrap();
    match cli_script.command.unwrap() {
        Commands::New(NewArgs { kind: NewKind::Script { name, out } }) => {
            assert_eq!(name, "deploy-token");
            assert!(out.is_none());
        }
        _ => panic!("Expected new script"),
    }

    let cli_contract = Cli::try_parse_from(["para", "new", "contract", "Vault"]).unwrap();
    match cli_contract.command.unwrap() {
        Commands::New(NewArgs { kind: NewKind::Contract { name, out } }) => {
            assert_eq!(name, "Vault");
            assert!(out.is_none());
        }
        _ => panic!("Expected new contract"),
    }
}

#[test]
fn test_cli_gui_options() {
    let cli = Cli::try_parse_from(["para", "gui", "-p", "8080", "--no-open"]).unwrap();
    match cli.command.unwrap() {
        Commands::Gui(rusty_chain_kit::cli::GuiArgs { port, no_open }) => {
            assert_eq!(port, 8080);
            assert!(no_open);
        }
        _ => panic!("Expected gui command"),
    }
}

#[test]
fn test_cli_passthrough_commands() {
    let forge = Cli::try_parse_from(["para", "forge", "build", "--sizes"]).unwrap();
    match forge.command.unwrap() {
        Commands::Forge { args } => assert_eq!(args, vec!["build", "--sizes"]),
        _ => panic!("Expected forge passthrough"),
    }

    let cast = Cli::try_parse_from(["para", "cast", "call", "0x0", "balance()"]).unwrap();
    match cast.command.unwrap() {
        Commands::Cast { args } => assert_eq!(args, vec!["call", "0x0", "balance()"]),
        _ => panic!("Expected cast passthrough"),
    }

    let anvil = Cli::try_parse_from(["para", "anvil", "--port", "9545"]).unwrap();
    match anvil.command.unwrap() {
        Commands::Anvil { args } => assert_eq!(args, vec!["--port", "9545"]),
        _ => panic!("Expected anvil passthrough"),
    }

    let solar = Cli::try_parse_from(["para", "solar", "--emit", "abi", "Foo.sol"]).unwrap();
    match solar.command.unwrap() {
        Commands::Solar { args } => assert_eq!(args, vec!["--emit", "abi", "Foo.sol"]),
        _ => panic!("Expected solar passthrough"),
    }
}

#[test]
fn test_cli_fallback_passthrough() {
    let cli = Cli::try_parse_from(["para", "custom-binary", "--arg1", "val1"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.passthrough, vec!["custom-binary", "--arg1", "val1"]);
}
