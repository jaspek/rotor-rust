use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;

use rotor::config::{Config, Xlen};
use rotor::model::generator;

#[derive(Parser, Debug)]
#[command(
    name = "rotor",
    about = "BTOR2 model generator for RISC-V machines",
    long_about = "Rotor generates bit-precise BTOR2 models of RISC-V machines \
                  for formal verification via bounded model checking.\n\n\
                  Supports RV64I/RV32I with M (multiply/divide) and C (compressed) extensions."
)]
struct Cli {
    /// RISC-V ELF binary to model
    #[arg(value_name = "BINARY")]
    binary: Option<PathBuf>,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Target architecture: x32 or x64
    #[arg(short = 'x', long, default_value = "x64")]
    xlen: Xlen,

    /// Enable M extension (multiply/divide)
    #[arg(long, default_value_t = true)]
    enable_m: bool,

    /// Enable C extension (compressed instructions)
    #[arg(long, default_value_t = true)]
    enable_c: bool,

    /// Number of cores to model
    #[arg(long, default_value_t = 1)]
    cores: usize,

    /// Virtual address space bits
    #[arg(long, default_value_t = 32)]
    vaddr_bits: u32,

    /// Heap allowance in bytes
    #[arg(long, default_value_t = 4096)]
    heap: u64,

    /// Stack allowance in bytes
    #[arg(long, default_value_t = 4096)]
    stack: u64,

    /// Bytes available for reading from stdin
    #[arg(long, default_value_t = 4)]
    bytes_to_read: u64,

    /// Check for bad exit code (exit != 0)
    #[arg(long, default_value_t = true)]
    check_bad_exit: bool,

    /// Check for good exit code (exit == 0)
    #[arg(long)]
    check_good_exit: bool,

    /// Check for any exit
    #[arg(long)]
    check_exit: bool,

    /// Check for division by zero
    #[arg(long, default_value_t = true)]
    check_div_zero: bool,

    /// Check for segmentation faults
    #[arg(long, default_value_t = true)]
    check_seg_faults: bool,

    /// Include comments in BTOR2 output
    #[arg(long, default_value_t = true)]
    comments: bool,

    /// Enable symbolic command-line arguments (argv)
    #[arg(long)]
    symbolic_argv: bool,

    /// Number of symbolic arguments (requires --symbolic-argv)
    #[arg(long = "num-symbolic-args", alias = "symbolic-argc", default_value_t = 1)]
    num_symbolic_args: usize,

    /// Maximum length of each symbolic argument in bytes (requires --symbolic-argv)
    #[arg(long, default_value_t = 8)]
    max_arglen: usize,

    /// Code synthesis mode (no binary needed)
    #[arg(long)]
    synthesis: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    if std::env::args().any(|a| a == "--symbolic-argc") {
        eprintln!(
            "warning: --symbolic-argc is deprecated, use --num-symbolic-args instead"
        );
    }

    let config = Config {
        xlen: cli.xlen,
        enable_m: cli.enable_m,
        enable_c: cli.enable_c,
        riscu_only: false,
        num_cores: cli.cores,
        virtual_address_space: cli.vaddr_bits,
        code_word_size: 32,
        heap_allowance: cli.heap,
        stack_allowance: cli.stack,
        bytes_to_read: cli.bytes_to_read,
        symbolic_argv: cli.symbolic_argv,
        symbolic_argc: cli.num_symbolic_args,
        max_arglen: cli.max_arglen,
        check_bad_exit_code: cli.check_bad_exit,
        check_good_exit_code: cli.check_good_exit,
        check_exit_codes: cli.check_exit,
        check_division_by_zero: cli.check_div_zero,
        check_division_overflow: true,
        check_seg_faults: cli.check_seg_faults,
        print_comments: cli.comments,
        propagate_constants: true,
    };

    let mut output: Box<dyn Write> = match &cli.output {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };

    if cli.synthesis {
        generator::model_rotor_synthesis(&config, &mut output)?;
    } else {
        let binary_path = cli
            .binary
            .ok_or("Binary path required (use --synthesis for code synthesis mode)")?;
        generator::model_rotor(&binary_path, &config, &mut output)?;
    }

    Ok(())
}
