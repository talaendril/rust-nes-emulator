#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq)]
pub enum InterruptType {
    NMI,
    BRK,
}

// pub(super) might be useless here, I kept it because it is something new I haven't seen before.
// If you were to use multiple modules inside of each other, you could use this pub(super) to make
// the struct, function or whatever you are using it with, visible inside the parent module
pub(super) struct Interrupt {
    pub(super) _itype: InterruptType,
    pub(super) vector_addr: u16,
    pub(super) b_flag_mask: u8,
    pub(super) cpu_cycles: u8,
}

pub(super) const NMI: Interrupt = Interrupt {
    _itype: InterruptType::NMI,
    vector_addr: 0xFFFA,
    b_flag_mask: 0b0010_0000,
    cpu_cycles: 2,
};

pub(super) const BRK: Interrupt = Interrupt {
    _itype: InterruptType::BRK,
    vector_addr: 0xFFFE,
    b_flag_mask: 0b0011_0000,
    cpu_cycles: 1,
};
