use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Xlen {
    X32,
    X64,
}

impl Xlen {
    pub fn bits(self) -> u32 {
        match self {
            Xlen::X32 => 32,
            Xlen::X64 => 64,
        }
    }

    pub fn bytes(self) -> u32 {
        self.bits() / 8
    }

    pub fn double_bits(self) -> u32 {
        self.bits() * 2
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub xlen: Xlen,
    pub enable_m: bool,
    pub enable_c: bool,
    pub riscu_only: bool,
    pub num_cores: usize,
    pub virtual_address_space: u32,
    pub code_word_size: u32,
    pub heap_allowance: u64,
    pub stack_allowance: u64,
    pub bytes_to_read: u64,
    // property checks
    pub check_bad_exit_code: bool,
    pub check_good_exit_code: bool,
    pub check_exit_codes: bool,
    pub check_division_by_zero: bool,
    pub check_division_overflow: bool,
    pub check_seg_faults: bool,
    // output
    pub print_comments: bool,
    pub propagate_constants: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            xlen: Xlen::X64,
            enable_m: true,
            enable_c: true,
            riscu_only: false,
            num_cores: 1,
            virtual_address_space: 32,
            code_word_size: 32,
            heap_allowance: 4096,
            stack_allowance: 4096,
            bytes_to_read: 4,
            check_bad_exit_code: true,
            check_good_exit_code: false,
            check_exit_codes: false,
            check_division_by_zero: true,
            check_division_overflow: true,
            check_seg_faults: true,
            print_comments: true,
            propagate_constants: true,
        }
    }
}

impl Config {
    pub fn machine_word_bits(&self) -> u32 {
        self.xlen.bits()
    }

    pub fn machine_word_bytes(&self) -> u32 {
        self.xlen.bytes()
    }

    pub fn instruction_size(&self) -> u32 {
        if self.enable_c { 2 } else { 4 }
    }
}
