use std::io::Write;
use std::path::Path;

use crate::btor2::builder::Btor2Builder;
use crate::btor2::printer::Btor2Printer;
use crate::config::Config;
use crate::machine::core::CoreState;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::model::combinational::rotor_combinational;
use crate::model::properties::rotor_properties;
use crate::model::sequential::rotor_sequential;
use crate::riscv::elf_loader::{self, LoadedBinary};

/// Top-level model generation pipeline.
pub fn model_rotor(
    binary_path: &Path,
    config: &Config,
    output: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Loading ELF binary: {:?}", binary_path);
    let binary = elf_loader::load_elf(binary_path)?;

    log::info!(
        "Binary: {} ({}bit), entry=0x{:x}, code={} bytes, data={} bytes",
        binary.name,
        if binary.is_64bit { 64 } else { 32 },
        binary.entry_point,
        binary.code_size,
        binary.data_size,
    );

    let mut builder = Btor2Builder::new();

    // Phase 1: Initialize sorts and constants
    log::info!("Initializing sorts and constants...");
    let sorts = MachineSorts::new(&mut builder, config);
    let consts = MachineConstants::new(&mut builder, &sorts, config);

    // Phase 2: Create per-core state
    log::info!("Creating core state ({} core(s))...", config.num_cores);
    let mut cores = Vec::new();
    for core_id in 0..config.num_cores {
        let core = CoreState::new(&mut builder, &sorts, &consts, config, &binary, core_id);
        cores.push(core);
    }

    // Phase 3: Generate combinational and sequential logic per core
    for core in &cores {
        log::info!("Generating logic for core {}...", core.core_id);

        // Combinational: fetch, decode, data flow, control flow
        let comb = rotor_combinational(&mut builder, &sorts, &consts, config, core);

        // Sequential: next-state for PC, registers, memory
        rotor_sequential(&mut builder, &sorts, &consts, config, core, &comb);

        // Properties: bad states
        rotor_properties(&mut builder, &sorts, &consts, config, core, &comb);
    }

    // Phase 4: Print BTOR2 model
    log::info!("Printing BTOR2 model ({} nodes)...", builder.node_count());
    let printer = Btor2Printer::new(config.print_comments);
    printer.print(&builder, &mut *output)?;

    log::info!("Done.");
    Ok(())
}

/// Generate a model without loading a binary (for code synthesis).
pub fn model_rotor_synthesis(
    config: &Config,
    output: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Btor2Builder::new();

    let sorts = MachineSorts::new(&mut builder, config);
    let consts = MachineConstants::new(&mut builder, &sorts, config);

    // For synthesis: create a core with symbolic code segment (no binary loaded).
    // This is a simplified version — the code segment remains an unconstrained input.
    // A full implementation would create symbolic instruction inputs.

    log::info!("Code synthesis mode — generating model with symbolic code...");

    // Create a minimal "empty" binary for the core state
    let empty_binary = LoadedBinary {
        name: "synthesis".to_string(),
        entry_point: 0,
        code: vec![0; 4], // single NOP
        code_start: 0,
        code_size: 4,
        data: vec![],
        data_start: 0x1000,
        data_size: 0,
        is_64bit: config.xlen == crate::config::Xlen::X64,
    };

    for core_id in 0..config.num_cores {
        let core = CoreState::new(
            &mut builder,
            &sorts,
            &consts,
            config,
            &empty_binary,
            core_id,
        );
        let comb = rotor_combinational(&mut builder, &sorts, &consts, config, &core);
        rotor_sequential(&mut builder, &sorts, &consts, config, &core, &comb);
        rotor_properties(&mut builder, &sorts, &consts, config, &core, &comb);
    }

    let printer = Btor2Printer::new(config.print_comments);
    printer.print(&builder, &mut *output)?;

    Ok(())
}
