//! (TODO!)

use super::{ERROR_ON_ACV_SETTING_ADDR, USER_PROG_START_ADDR, traps as t};
use super::{OS_DEFAULT_STARTING_SP, OS_STARTING_SP_ADDR};

use lc3_isa::util::{AssembledProgram, MemoryDump};
use lc3_isa::{Word, OS_START_ADDR};
use lc3_baseline_sim::{KBSR_ADDR, KBDR_ADDR, DSR_ADDR, DDR_ADDR};
use lc3_baseline_sim::{G0CR_ADDR, A0CR_ADDR, P0CR_ADDR, T0CR_ADDR, CLKR_ADDR};
use lc3_baseline_sim::{GPIO_OFFSET, ADC_OFFSET, PWM_OFFSET, TIMER_OFFSET, MISC_OFFSET};
use lc3_baseline_sim::{GPIO_BASE_INT_VEC, TIMER_BASE_INT_VEC};

use lazy_static::lazy_static;

lazy_static! {
    /// The LC-3 OS this crate provides in [MemoryDump](lc3_isa::util::MemoryDump) form.
    pub static ref OS_IMAGE: MemoryDump = os().into();

    /// The LC-3 OS this crate provides in raw
    /// ([AssembledProgram](lc3_isa::util::AssembledProgram)) form. This can be turned
    /// into a (Addr, Word) iterator over *only the words that are set.
    pub static ref OS: AssembledProgram = os();
}

#[cfg(feature = "nightly-const")]
pub const CONST_OS: AssembledProgram = os();

nightly_const! { [] => [
fn os() -> AssembledProgram {
    use Word as W;

    let os = lc3_isa::program! {
        // The following is a lightly modified version of the OS that ships with Chirag
        // Sakhuja's [lc3tools](https://github.com/chiragsakhuja/lc3tools). Many thanks
        // to him and the other contributors to the lc3tools project.
        //
        // ([link to the original](https://github.com/chiragsakhuja/lc3tools/blob/b5d7245aabc33a05f28cc124202fd1532b1d9609/backend/lc3os.cpp#L12-L673))

        .ORIG #0x0000;

        //// The TRAP vector table (0x0000 - 0x00FF) ////
        .FILL @UNKNOWN_TRAP; // 0x00
        .FILL @UNKNOWN_TRAP; // 0x01
        .FILL @UNKNOWN_TRAP; // 0x02
        .FILL @UNKNOWN_TRAP; // 0x03
        .FILL @UNKNOWN_TRAP; // 0x04
        .FILL @UNKNOWN_TRAP; // 0x05
        .FILL @UNKNOWN_TRAP; // 0x06
        .FILL @UNKNOWN_TRAP; // 0x07
        .FILL @UNKNOWN_TRAP; // 0x08
        .FILL @UNKNOWN_TRAP; // 0x09
        .FILL @UNKNOWN_TRAP; // 0x0A
        .FILL @UNKNOWN_TRAP; // 0x0B
        .FILL @UNKNOWN_TRAP; // 0x0C
        .FILL @UNKNOWN_TRAP; // 0x0D
        .FILL @UNKNOWN_TRAP; // 0x0E
        .FILL @UNKNOWN_TRAP; // 0x0F
        .FILL @UNKNOWN_TRAP; // 0x10
        .FILL @UNKNOWN_TRAP; // 0x11
        .FILL @UNKNOWN_TRAP; // 0x12
        .FILL @UNKNOWN_TRAP; // 0x13
        .FILL @UNKNOWN_TRAP; // 0x14
        .FILL @UNKNOWN_TRAP; // 0x15
        .FILL @UNKNOWN_TRAP; // 0x16
        .FILL @UNKNOWN_TRAP; // 0x17
        .FILL @UNKNOWN_TRAP; // 0x18
        .FILL @UNKNOWN_TRAP; // 0x19
        .FILL @UNKNOWN_TRAP; // 0x1A
        .FILL @UNKNOWN_TRAP; // 0x1B
        .FILL @UNKNOWN_TRAP; // 0x1C
        .FILL @UNKNOWN_TRAP; // 0x1D
        .FILL @UNKNOWN_TRAP; // 0x1E
        .FILL @UNKNOWN_TRAP; // 0x1F

        .ORIG #t::builtin::GETC      as W;  .FILL @TRAP_GETC;                   // 0x20
        .ORIG #t::builtin::OUT       as W;  .FILL @TRAP_OUT;                    // 0x21
        .ORIG #t::builtin::PUTS      as W;  .FILL @TRAP_PUTS;                   // 0x22
        .ORIG #t::builtin::IN        as W;  .FILL @TRAP_IN;                     // 0x23
        .ORIG #t::builtin::PUTSP     as W;  .FILL @TRAP_PUTSP;                  // 0x24
        .ORIG #t::builtin::HALT      as W;  .FILL @TRAP_HALT;                   // 0x25
        .FILL @UNKNOWN_TRAP; // 0x26
        .FILL @UNKNOWN_TRAP; // 0x27
        .FILL @UNKNOWN_TRAP; // 0x28
        .FILL @UNKNOWN_TRAP; // 0x29
        .FILL @UNKNOWN_TRAP; // 0x2A
        .FILL @UNKNOWN_TRAP; // 0x2B
        .FILL @UNKNOWN_TRAP; // 0x2C
        .FILL @UNKNOWN_TRAP; // 0x2D
        .FILL @UNKNOWN_TRAP; // 0x2E
        .FILL @UNKNOWN_TRAP; // 0x2F

        .ORIG #GPIO_OFFSET as Word;
        .ORIG #t::gpio::INPUT        as W;  .FILL @TRAP_SET_GPIO_INPUT;         // 0x30
        .ORIG #t::gpio::OUTPUT       as W;  .FILL @TRAP_SET_GPIO_OUTPUT;        // 0x31
        .ORIG #t::gpio::INTERRUPT    as W;  .FILL @TRAP_SET_GPIO_INTERRUPT;     // 0x32
        .ORIG #t::gpio::DISABLED     as W;  .FILL @TRAP_SET_GPIO_DISABLED;      // 0x33
        .ORIG #t::gpio::GET_MODE     as W;  .FILL @TRAP_READ_GPIO_MODE;         // 0x34
        .ORIG #t::gpio::WRITE        as W;  .FILL @TRAP_WRITE_GPIO_DATA;        // 0x35
        .ORIG #t::gpio::READ         as W;  .FILL @TRAP_READ_GPIO_DATA;         // 0x36
        .FILL @UNKNOWN_TRAP; // 0x37
        .FILL @UNKNOWN_TRAP; // 0x38
        .FILL @UNKNOWN_TRAP; // 0x39
        .FILL @UNKNOWN_TRAP; // 0x3A
        .FILL @UNKNOWN_TRAP; // 0x3B
        .FILL @UNKNOWN_TRAP; // 0x3C
        .FILL @UNKNOWN_TRAP; // 0x3D
        .FILL @UNKNOWN_TRAP; // 0x3E
        .FILL @UNKNOWN_TRAP; // 0x3F

        .ORIG #ADC_OFFSET as Word;
        .ORIG #t::adc::ENABLE        as W;  .FILL @TRAP_SET_ADC_ENABLE;         // 0x40
        .ORIG #t::adc::DISABLE       as W;  .FILL @TRAP_SET_ADC_DISABLE;        // 0x41
        .ORIG #t::adc::GET_MODE      as W;  .FILL @TRAP_READ_ADC_MODE;          // 0x42
        .ORIG #t::adc::READ          as W;  .FILL @TRAP_READ_ADC_DATA;          // 0x43
        .FILL @UNKNOWN_TRAP; // 0x44
        .FILL @UNKNOWN_TRAP; // 0x45
        .FILL @UNKNOWN_TRAP; // 0x46
        .FILL @UNKNOWN_TRAP; // 0x47
        .FILL @UNKNOWN_TRAP; // 0x48
        .FILL @UNKNOWN_TRAP; // 0x49
        .FILL @UNKNOWN_TRAP; // 0x4A
        .FILL @UNKNOWN_TRAP; // 0x4B
        .FILL @UNKNOWN_TRAP; // 0x4C
        .FILL @UNKNOWN_TRAP; // 0x4D
        .FILL @UNKNOWN_TRAP; // 0x4E
        .FILL @UNKNOWN_TRAP; // 0x4F

        .ORIG #PWM_OFFSET as Word;
        .ORIG #t::pwm::ENABLE        as W;  .FILL @TRAP_SET_PWM;                // 0x50
        .ORIG #t::pwm::DISABLE       as W;  .FILL @TRAP_DISABLE_PWM;            // 0x51
        .ORIG #t::pwm::GET_PERIOD    as W;  .FILL @TRAP_READ_PWM_PERIOD;        // 0x52
        .ORIG #t::pwm::GET_DUTY      as W;  .FILL @TRAP_READ_PWM_DUTY_CYCLE;    // 0x53
        .FILL @UNKNOWN_TRAP; // 0x54
        .FILL @UNKNOWN_TRAP; // 0x55
        .FILL @UNKNOWN_TRAP; // 0x56
        .FILL @UNKNOWN_TRAP; // 0x57
        .FILL @UNKNOWN_TRAP; // 0x58
        .FILL @UNKNOWN_TRAP; // 0x59
        .FILL @UNKNOWN_TRAP; // 0x5A
        .FILL @UNKNOWN_TRAP; // 0x5B
        .FILL @UNKNOWN_TRAP; // 0x5C
        .FILL @UNKNOWN_TRAP; // 0x5D
        .FILL @UNKNOWN_TRAP; // 0x5E
        .FILL @UNKNOWN_TRAP; // 0x5F

        .ORIG #TIMER_OFFSET as Word;
        .ORIG #t::timers::SINGLESHOT as W;  .FILL @TRAP_SET_TIMER_SINGLESHOT;   // 0x60
        .ORIG #t::timers::REPEATED   as W;  .FILL @TRAP_SET_TIMER_REPEAT;       // 0x61
        .ORIG #t::timers::DISABLE    as W;  .FILL @TRAP_SET_TIMER_DISABLE;      // 0x62
        .ORIG #t::timers::GET_MODE   as W;  .FILL @TRAP_READ_TIMER_MODE;        // 0x63
        .ORIG #t::timers::GET_PERIOD as W;  .FILL @TRAP_READ_TIMER_PERIOD;      // 0x64
        .FILL @UNKNOWN_TRAP; // 0x65
        .FILL @UNKNOWN_TRAP; // 0x66
        .FILL @UNKNOWN_TRAP; // 0x67
        .FILL @UNKNOWN_TRAP; // 0x68
        .FILL @UNKNOWN_TRAP; // 0x69
        .FILL @UNKNOWN_TRAP; // 0x6A
        .FILL @UNKNOWN_TRAP; // 0x6B
        .FILL @UNKNOWN_TRAP; // 0x6C
        .FILL @UNKNOWN_TRAP; // 0x6D
        .FILL @UNKNOWN_TRAP; // 0x6E
        .FILL @UNKNOWN_TRAP; // 0x6F

        .ORIG #MISC_OFFSET as Word;
        .ORIG #t::clock::SET         as W;  .FILL @TRAP_SET_CLOCK;              // 0x70
        .ORIG #t::clock::GET         as W;  .FILL @TRAP_READ_CLOCK;             // 0x71
        .FILL @UNKNOWN_TRAP; // 0x72
        .FILL @UNKNOWN_TRAP; // 0x73
        .FILL @UNKNOWN_TRAP; // 0x74
        .FILL @UNKNOWN_TRAP; // 0x75
        .FILL @UNKNOWN_TRAP; // 0x76
        .FILL @UNKNOWN_TRAP; // 0x77
        .FILL @UNKNOWN_TRAP; // 0x78
        .FILL @UNKNOWN_TRAP; // 0x79
        .FILL @UNKNOWN_TRAP; // 0x7A
        .FILL @UNKNOWN_TRAP; // 0x7B
        .FILL @UNKNOWN_TRAP; // 0x7C
        .FILL @UNKNOWN_TRAP; // 0x7D
        .FILL @UNKNOWN_TRAP; // 0x7E
        .FILL @UNKNOWN_TRAP; // 0x7F

        .FILL @UNKNOWN_TRAP; // 0x80
        .FILL @UNKNOWN_TRAP; // 0x81
        .FILL @UNKNOWN_TRAP; // 0x82
        .FILL @UNKNOWN_TRAP; // 0x83
        .FILL @UNKNOWN_TRAP; // 0x84
        .FILL @UNKNOWN_TRAP; // 0x85
        .FILL @UNKNOWN_TRAP; // 0x86
        .FILL @UNKNOWN_TRAP; // 0x87
        .FILL @UNKNOWN_TRAP; // 0x88
        .FILL @UNKNOWN_TRAP; // 0x89
        .FILL @UNKNOWN_TRAP; // 0x8A
        .FILL @UNKNOWN_TRAP; // 0x8B
        .FILL @UNKNOWN_TRAP; // 0x8C
        .FILL @UNKNOWN_TRAP; // 0x8D
        .FILL @UNKNOWN_TRAP; // 0x8E
        .FILL @UNKNOWN_TRAP; // 0x8F
        .FILL @UNKNOWN_TRAP; // 0x90
        .FILL @UNKNOWN_TRAP; // 0x91
        .FILL @UNKNOWN_TRAP; // 0x92
        .FILL @UNKNOWN_TRAP; // 0x93
        .FILL @UNKNOWN_TRAP; // 0x94
        .FILL @UNKNOWN_TRAP; // 0x95
        .FILL @UNKNOWN_TRAP; // 0x96
        .FILL @UNKNOWN_TRAP; // 0x97
        .FILL @UNKNOWN_TRAP; // 0x98
        .FILL @UNKNOWN_TRAP; // 0x99
        .FILL @UNKNOWN_TRAP; // 0x9A
        .FILL @UNKNOWN_TRAP; // 0x9B
        .FILL @UNKNOWN_TRAP; // 0x9C
        .FILL @UNKNOWN_TRAP; // 0x9D
        .FILL @UNKNOWN_TRAP; // 0x9E
        .FILL @UNKNOWN_TRAP; // 0x9F
        .FILL @UNKNOWN_TRAP; // 0xA0
        .FILL @UNKNOWN_TRAP; // 0xA1
        .FILL @UNKNOWN_TRAP; // 0xA2
        .FILL @UNKNOWN_TRAP; // 0xA3
        .FILL @UNKNOWN_TRAP; // 0xA4
        .FILL @UNKNOWN_TRAP; // 0xA5
        .FILL @UNKNOWN_TRAP; // 0xA6
        .FILL @UNKNOWN_TRAP; // 0xA7
        .FILL @UNKNOWN_TRAP; // 0xA8
        .FILL @UNKNOWN_TRAP; // 0xA9
        .FILL @UNKNOWN_TRAP; // 0xAA
        .FILL @UNKNOWN_TRAP; // 0xAB
        .FILL @UNKNOWN_TRAP; // 0xAC
        .FILL @UNKNOWN_TRAP; // 0xAD
        .FILL @UNKNOWN_TRAP; // 0xAE
        .FILL @UNKNOWN_TRAP; // 0xAF
        .FILL @UNKNOWN_TRAP; // 0xB0
        .FILL @UNKNOWN_TRAP; // 0xB1
        .FILL @UNKNOWN_TRAP; // 0xB2
        .FILL @UNKNOWN_TRAP; // 0xB3
        .FILL @UNKNOWN_TRAP; // 0xB4
        .FILL @UNKNOWN_TRAP; // 0xB5
        .FILL @UNKNOWN_TRAP; // 0xB6
        .FILL @UNKNOWN_TRAP; // 0xB7
        .FILL @UNKNOWN_TRAP; // 0xB8
        .FILL @UNKNOWN_TRAP; // 0xB9
        .FILL @UNKNOWN_TRAP; // 0xBA
        .FILL @UNKNOWN_TRAP; // 0xBB
        .FILL @UNKNOWN_TRAP; // 0xBC
        .FILL @UNKNOWN_TRAP; // 0xBD
        .FILL @UNKNOWN_TRAP; // 0xBE
        .FILL @UNKNOWN_TRAP; // 0xBF
        .FILL @UNKNOWN_TRAP; // 0xC0
        .FILL @UNKNOWN_TRAP; // 0xC1
        .FILL @UNKNOWN_TRAP; // 0xC2
        .FILL @UNKNOWN_TRAP; // 0xC3
        .FILL @UNKNOWN_TRAP; // 0xC4
        .FILL @UNKNOWN_TRAP; // 0xC5
        .FILL @UNKNOWN_TRAP; // 0xC6
        .FILL @UNKNOWN_TRAP; // 0xC7
        .FILL @UNKNOWN_TRAP; // 0xC8
        .FILL @UNKNOWN_TRAP; // 0xC9
        .FILL @UNKNOWN_TRAP; // 0xCA
        .FILL @UNKNOWN_TRAP; // 0xCB
        .FILL @UNKNOWN_TRAP; // 0xCC
        .FILL @UNKNOWN_TRAP; // 0xCD
        .FILL @UNKNOWN_TRAP; // 0xCE
        .FILL @UNKNOWN_TRAP; // 0xCF
        .FILL @UNKNOWN_TRAP; // 0xD0
        .FILL @UNKNOWN_TRAP; // 0xD1
        .FILL @UNKNOWN_TRAP; // 0xD2
        .FILL @UNKNOWN_TRAP; // 0xD3
        .FILL @UNKNOWN_TRAP; // 0xD4
        .FILL @UNKNOWN_TRAP; // 0xD5
        .FILL @UNKNOWN_TRAP; // 0xD6
        .FILL @UNKNOWN_TRAP; // 0xD7
        .FILL @UNKNOWN_TRAP; // 0xD8
        .FILL @UNKNOWN_TRAP; // 0xD9
        .FILL @UNKNOWN_TRAP; // 0xDA
        .FILL @UNKNOWN_TRAP; // 0xDB
        .FILL @UNKNOWN_TRAP; // 0xDC
        .FILL @UNKNOWN_TRAP; // 0xDD
        .FILL @UNKNOWN_TRAP; // 0xDE
        .FILL @UNKNOWN_TRAP; // 0xDF
        .FILL @UNKNOWN_TRAP; // 0xE0
        .FILL @UNKNOWN_TRAP; // 0xE1
        .FILL @UNKNOWN_TRAP; // 0xE2
        .FILL @UNKNOWN_TRAP; // 0xE3
        .FILL @UNKNOWN_TRAP; // 0xE4
        .FILL @UNKNOWN_TRAP; // 0xE5
        .FILL @UNKNOWN_TRAP; // 0xE6
        .FILL @UNKNOWN_TRAP; // 0xE7
        .FILL @UNKNOWN_TRAP; // 0xE8
        .FILL @UNKNOWN_TRAP; // 0xE9
        .FILL @UNKNOWN_TRAP; // 0xEA
        .FILL @UNKNOWN_TRAP; // 0xEB
        .FILL @UNKNOWN_TRAP; // 0xEC
        .FILL @UNKNOWN_TRAP; // 0xED
        .FILL @UNKNOWN_TRAP; // 0xEE
        .FILL @UNKNOWN_TRAP; // 0xEF
        .FILL @UNKNOWN_TRAP; // 0xF0
        .FILL @UNKNOWN_TRAP; // 0xF1
        .FILL @UNKNOWN_TRAP; // 0xF2
        .FILL @UNKNOWN_TRAP; // 0xF3
        .FILL @UNKNOWN_TRAP; // 0xF4
        .FILL @UNKNOWN_TRAP; // 0xF5
        .FILL @UNKNOWN_TRAP; // 0xF6
        .FILL @UNKNOWN_TRAP; // 0xF7
        .FILL @UNKNOWN_TRAP; // 0xF8
        .FILL @UNKNOWN_TRAP; // 0xF9
        .FILL @UNKNOWN_TRAP; // 0xFA
        .FILL @UNKNOWN_TRAP; // 0xFB
        .FILL @UNKNOWN_TRAP; // 0xFC
        .FILL @UNKNOWN_TRAP; // 0xFD
        .FILL @UNKNOWN_TRAP; // 0xFE
        .FILL @UNKNOWN_TRAP; // 0xFF

        ////  The Exception vector table (0x0100 - 0x017F) ////
        // TODO: use constants in ISA?
        .FILL @PRIVILEGE_MODE_EX_HANDLER; // 0x100 -- TODO: only used for calling RTI when not in an interrupt
        .FILL @ILLEGAL_OPCODE_EX_HANDLER; // 0x101
        .FILL @ACV_EX_HANDLER;            // 0x102 -- TODO: verify
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x103
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x104
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x105
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x106
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x107
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x108
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x109
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x10A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x10B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x10C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x10D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x10E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x10F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x110
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x111
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x112
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x113
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x114
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x115
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x116
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x117
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x118
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x119
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x11A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x11B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x11C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x11D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x11E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x11F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x120
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x121
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x122
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x123
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x124
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x125
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x126
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x127
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x128
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x129
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x12A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x12B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x12C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x12D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x12E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x12F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x130
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x131
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x132
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x133
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x134
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x135
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x136
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x137
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x138
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x139
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x13A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x13B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x13C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x13D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x13E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x13F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x140
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x141
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x142
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x143
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x144
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x145
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x146
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x147
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x148
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x149
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x14A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x14B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x14C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x14D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x14E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x14F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x150
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x151
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x152
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x153
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x154
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x155
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x156
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x157
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x158
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x159
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x15A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x15B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x15C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x15D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x15E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x15F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x160
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x161
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x162
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x163
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x164
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x165
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x166
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x167
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x168
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x169
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x16A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x16B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x16C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x16D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x16E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x16F
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x170
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x171
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x172
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x173
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x174
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x175
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x176
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x177
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x178
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x179
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x17A
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x17B
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x17C
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x17D
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x17E
        .FILL @DEFAULT_EXCEPTION_HANDLER; // 0x17F

        //// The Interrupt vector table (0x0180 - 0x01FF) ////
        .FILL @DEFAULT_INT_HANDLER; // 0x180: Keyboard Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x181: Display Interrupt (TODO: verify)
        .FILL @DEFAULT_INT_HANDLER; // 0x182
        .FILL @DEFAULT_INT_HANDLER; // 0x183
        .FILL @DEFAULT_INT_HANDLER; // 0x184
        .FILL @DEFAULT_INT_HANDLER; // 0x185
        .FILL @DEFAULT_INT_HANDLER; // 0x186
        .FILL @DEFAULT_INT_HANDLER; // 0x187
        .FILL @DEFAULT_INT_HANDLER; // 0x188
        .FILL @DEFAULT_INT_HANDLER; // 0x189
        .FILL @DEFAULT_INT_HANDLER; // 0x18A
        .FILL @DEFAULT_INT_HANDLER; // 0x18B
        .FILL @DEFAULT_INT_HANDLER; // 0x18C
        .FILL @DEFAULT_INT_HANDLER; // 0x18D
        .FILL @DEFAULT_INT_HANDLER; // 0x18E
        .FILL @DEFAULT_INT_HANDLER; // 0x18F
        .FILL @DEFAULT_INT_HANDLER; // 0x190
        .FILL @DEFAULT_INT_HANDLER; // 0x191
        .FILL @DEFAULT_INT_HANDLER; // 0x192
        .FILL @DEFAULT_INT_HANDLER; // 0x193
        .FILL @DEFAULT_INT_HANDLER; // 0x194
        .FILL @DEFAULT_INT_HANDLER; // 0x195
        .FILL @DEFAULT_INT_HANDLER; // 0x196
        .FILL @DEFAULT_INT_HANDLER; // 0x197
        .FILL @DEFAULT_INT_HANDLER; // 0x198
        .FILL @DEFAULT_INT_HANDLER; // 0x199
        .FILL @DEFAULT_INT_HANDLER; // 0x19A
        .FILL @DEFAULT_INT_HANDLER; // 0x19B
        .FILL @DEFAULT_INT_HANDLER; // 0x19C
        .FILL @DEFAULT_INT_HANDLER; // 0x19D
        .FILL @DEFAULT_INT_HANDLER; // 0x19E
        .FILL @DEFAULT_INT_HANDLER; // 0x19F
        .FILL @DEFAULT_INT_HANDLER; // 0x1A0
        .FILL @DEFAULT_INT_HANDLER; // 0x1A1
        .FILL @DEFAULT_INT_HANDLER; // 0x1A2
        .FILL @DEFAULT_INT_HANDLER; // 0x1A3
        .FILL @DEFAULT_INT_HANDLER; // 0x1A4
        .FILL @DEFAULT_INT_HANDLER; // 0x1A5
        .FILL @DEFAULT_INT_HANDLER; // 0x1A6
        .FILL @DEFAULT_INT_HANDLER; // 0x1A7
        .FILL @DEFAULT_INT_HANDLER; // 0x1A8
        .FILL @DEFAULT_INT_HANDLER; // 0x1A9
        .FILL @DEFAULT_INT_HANDLER; // 0x1AA
        .FILL @DEFAULT_INT_HANDLER; // 0x1AB
        .FILL @DEFAULT_INT_HANDLER; // 0x1AC
        .FILL @DEFAULT_INT_HANDLER; // 0x1AD
        .FILL @DEFAULT_INT_HANDLER; // 0x1AE
        .FILL @DEFAULT_INT_HANDLER; // 0x1AF
        .FILL @DEFAULT_INT_HANDLER; // 0x1B0: G0 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B1: G1 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B2: G2 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B3: G3 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B4: G4 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B5: G5 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B6: G6 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B7: G7 Interrupt
        .FILL @DEFAULT_INT_HANDLER; // 0x1B8
        .FILL @DEFAULT_INT_HANDLER; // 0x1B9
        .FILL @DEFAULT_INT_HANDLER; // 0x1BA
        .FILL @DEFAULT_INT_HANDLER; // 0x1BB
        .FILL @DEFAULT_INT_HANDLER; // 0x1BC
        .FILL @DEFAULT_INT_HANDLER; // 0x1BD
        .FILL @DEFAULT_INT_HANDLER; // 0x1BE
        .FILL @DEFAULT_INT_HANDLER; // 0x1BF
        .FILL @DEFAULT_INT_HANDLER; // 0x1C0
        .FILL @DEFAULT_INT_HANDLER; // 0x1C1
        .FILL @DEFAULT_INT_HANDLER; // 0x1C2
        .FILL @DEFAULT_INT_HANDLER; // 0x1C3
        .FILL @DEFAULT_INT_HANDLER; // 0x1C4
        .FILL @DEFAULT_INT_HANDLER; // 0x1C5
        .FILL @DEFAULT_INT_HANDLER; // 0x1C6
        .FILL @DEFAULT_INT_HANDLER; // 0x1C7
        .FILL @DEFAULT_INT_HANDLER; // 0x1C8
        .FILL @DEFAULT_INT_HANDLER; // 0x1C9
        .FILL @DEFAULT_INT_HANDLER; // 0x1CA
        .FILL @DEFAULT_INT_HANDLER; // 0x1CB
        .FILL @DEFAULT_INT_HANDLER; // 0x1CC
        .FILL @DEFAULT_INT_HANDLER; // 0x1CD
        .FILL @DEFAULT_INT_HANDLER; // 0x1CE
        .FILL @DEFAULT_INT_HANDLER; // 0x1CF
        .FILL @DEFAULT_INT_HANDLER; // 0x1D0
        .FILL @DEFAULT_INT_HANDLER; // 0x1D1
        .FILL @DEFAULT_INT_HANDLER; // 0x1D2
        .FILL @DEFAULT_INT_HANDLER; // 0x1D3
        .FILL @DEFAULT_INT_HANDLER; // 0x1D4
        .FILL @DEFAULT_INT_HANDLER; // 0x1D5
        .FILL @DEFAULT_INT_HANDLER; // 0x1D6
        .FILL @DEFAULT_INT_HANDLER; // 0x1D7
        .FILL @DEFAULT_INT_HANDLER; // 0x1D8
        .FILL @DEFAULT_INT_HANDLER; // 0x1D9
        .FILL @DEFAULT_INT_HANDLER; // 0x1DA
        .FILL @DEFAULT_INT_HANDLER; // 0x1DB
        .FILL @DEFAULT_INT_HANDLER; // 0x1DC
        .FILL @DEFAULT_INT_HANDLER; // 0x1DD
        .FILL @DEFAULT_INT_HANDLER; // 0x1DE
        .FILL @DEFAULT_INT_HANDLER; // 0x1DF
        .FILL @DEFAULT_INT_HANDLER; // 0x1E0: Timer 0 Interrupt (TODO)
        .FILL @DEFAULT_INT_HANDLER; // 0x1E1: Timer 1 Interrupt (TODO)
        .FILL @DEFAULT_INT_HANDLER; // 0x1E2
        .FILL @DEFAULT_INT_HANDLER; // 0x1E3
        .FILL @DEFAULT_INT_HANDLER; // 0x1E4
        .FILL @DEFAULT_INT_HANDLER; // 0x1E5
        .FILL @DEFAULT_INT_HANDLER; // 0x1E6
        .FILL @DEFAULT_INT_HANDLER; // 0x1E7
        .FILL @DEFAULT_INT_HANDLER; // 0x1E8
        .FILL @DEFAULT_INT_HANDLER; // 0x1E9
        .FILL @DEFAULT_INT_HANDLER; // 0x1EA
        .FILL @DEFAULT_INT_HANDLER; // 0x1EB
        .FILL @DEFAULT_INT_HANDLER; // 0x1EC
        .FILL @DEFAULT_INT_HANDLER; // 0x1ED
        .FILL @DEFAULT_INT_HANDLER; // 0x1EE
        .FILL @DEFAULT_INT_HANDLER; // 0x1EF
        .FILL @DEFAULT_INT_HANDLER; // 0x1F0
        .FILL @DEFAULT_INT_HANDLER; // 0x1F1
        .FILL @DEFAULT_INT_HANDLER; // 0x1F2
        .FILL @DEFAULT_INT_HANDLER; // 0x1F3
        .FILL @DEFAULT_INT_HANDLER; // 0x1F4
        .FILL @DEFAULT_INT_HANDLER; // 0x1F5
        .FILL @DEFAULT_INT_HANDLER; // 0x1F6
        .FILL @DEFAULT_INT_HANDLER; // 0x1F7
        .FILL @DEFAULT_INT_HANDLER; // 0x1F8
        .FILL @DEFAULT_INT_HANDLER; // 0x1F9
        .FILL @DEFAULT_INT_HANDLER; // 0x1FA
        .FILL @DEFAULT_INT_HANDLER; // 0x1FB
        .FILL @DEFAULT_INT_HANDLER; // 0x1FC
        .FILL @DEFAULT_INT_HANDLER; // 0x1FD
        .FILL @DEFAULT_INT_HANDLER; // 0x1FE
        .FILL @DEFAULT_INT_HANDLER; // 0x1FF


        //// OS Startup Routine ////
        .ORIG #OS_START_ADDR;
            LDI R6, @OS_STARTING_SP_PTR;          // Set the system stack pointer (SSP)

            LEA R0, @OS_START_MSG;  // Print a welcome message
            PUTS;

            // The below is different from the original lc3tools OS; unlike its version
            // of the startup routine, we do not hand control back to the simulator in
            // order to start executing the user program; instead we use RTI to set the
            // PSR and PC to do so. Many thanks to Steven Zhu ([@ss-couchpotato](https://github.com/ss-couchpotato))
            // for pointing out this approach.
            //
            // This has the benefit of not needing special logic* in the simulator to
            // start the user program, but also means that the OS startup routine must
            // either hardcode a fixed starting address or grow logic to handle variable
            // starting locations as lc3tools does.
            //
            // For now we handle this by having the startup routine grab the starting
            // address from a set memory location. By default this is 0x3000 and
            // binaries (which _include_ the OS) can override this if required.
            //
            // *: Ordinarily, setting the PSR manually (as the original lc3tools OS
            // startup routine does) and then calling HALT will trigger an ACV (upon
            // trying to fetch the TRAP instruction which resides in system space --
            // inaccessible to the now in user mode machine). Further while trying to
            // handle the ACV the machine will encounter yet another error as it
            // attempts to push the PSR and PC onto the system stack (which, since the
            // BSP was not set by the OS startup routine, is presumably 0) to prepare
            // to invoke the exception handler for access control violations: since the
            // system is in user mode, the BSP will be used as the system stack pointer
            // and since it is likely 0, a stack overflow will occur.

            // Prepare to switch to the user program:
            LD R0, @USER_PROG_PSR_INIT; // Fetch the initial PSR for the user program:

            ADD R6, R6, #-1;            // And then go push it onto the stack.
            STR R0, R6, #0;

            LDI R0, @USER_PROG_START_ADDR_PTR; // Fetch the starting address of the program.
            ADD R6, R6, #-1;            // And push that onto the stack next.
            STR R0, R6, #0;

            // Finally start the program!
            RTI;


        //// Constants ////
        @OS_START_MSG // ""
            .FILL #('\0' as Word);

        @USER_PROG_START_ADDR_PTR .FILL #USER_PROG_START_ADDR;

        @OS_STARTING_SP_PTR .FILL #OS_STARTING_SP_ADDR;

        @KBSR .FILL #KBSR_ADDR;
        @KBDR .FILL #KBDR_ADDR;
        @DSR .FILL #DSR_ADDR;
        @DDR .FILL #DDR_ADDR;

        @MCR .FILL #lc3_isa::MCR;

        @USER_PROG_PSR_INIT
            // { user_mode = true
            // , priority_level = 3
            // , n = false
            // , z = true
            // , p = false
            // }
            .FILL #0b1_0000_011_00000_0_1_0;
            //       |       |        | | \
            //       |       |        | \  p bit
            //       |       |        \  z bit
            //       |       \        n bit
            //       \   priority level (3)
            //   user mode

        @MASK_HI_BIT .FILL #0x7FFF;
        @MASK_LOW_BYTE .FILL #0x00FF;

        //// TRAP Routines ////

        // GETC: Gets a single character.
        //
        // Returns the character in R0.
        @TRAP_GETC
            LDI R0, @KBSR;
            BRzp @TRAP_GETC;  // Spin until there's a new character.

            LDI R0, @KBDR;    // When there is, read it in and return.
            RTI;

        // OUT: Outputs a single character.
        //
        // Takes the character to be printed in R0.
        @TRAP_OUT
            ADD R6, R6, #-1;
            STR R1, R6, #0;     // Save R1.

            @TRAP_OUT_WAIT
                LDI R1, @DSR;
                BRzp @TRAP_OUT_WAIT; // Spin until the display is ready.

            STI R0, @DDR;        // When it's ready, write the new character..
            LDR R1, R6, #0;
            ADD R6, R6, #1;      // ..restore R1..
            RTI;                 // ..and return.

        // PUTS: Outputs a string (null-terminated).
        //
        // Takes a pointer to a null-terminated string in R0.
        @TRAP_PUTS
            ADD R6, R6, #-2;       // Save R0 and R1
            STR R0, R6, #1;
            STR R1, R6, #0;
            ADD R1, R0, #0;        // Move string pointer (R0) into R1
        @TRAP_PUTS_LOOP
            LDR R0, R1, #0;        // Write characters in string using OUT
            BRz @TRAP_PUTS_DONE;
            OUT;
            ADD R1, R1, #1;
            BRnzp @TRAP_PUTS_LOOP;
        @TRAP_PUTS_DONE
            LDR R1, R6, #0;        // Restore R0 and R1
            LDR R0, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // IN: Outputs a prompt and reads in a character.
        //
        // Returns the character in R0.
        @TRAP_IN
            LEA R0, @TRAP_IN_MSG; // Output the prompt.
            PUTS;

            GETC;                 // Get the character.
            OUT;                  // Echo it.

            ADD R6, R6, #-1;
            STR R0, R6, #0;       // Save the character, print a newline.
            AND R0, R0, #0;
            ADD R0, R0, #('\n' as lc3_isa::SignedWord);
            OUT;

            LDR R0, R6, #0;
            ADD R6, R6, #1;       // Restore and return.
            RTI;

        // PUTSP: Output a packed (2 characters to a word) string.
        //
        // Note: As with lc3tools' implementation of this function, this will end when
        // it encounters a single NULL (an aligned double NULL isn't required).
        //
        // Takes a pointer to a string in R0.
        @TRAP_PUTSP
            ADD R6, R6, #-4;         // Save R0, R1, R2, and R3
            STR R0, R6, #3;
            STR R1, R6, #2;
            STR R2, R6, #1;
            STR R3, R6, #0;

            ADD R1, R0, #0;             // Copy over the string pointer (R0 -> R1).

            @TRAP_PUTSP_LOOP
                LDR R2, R1, #0;         // Read two characters.
                LD R0, @MASK_LOW_BYTE;  // Extract the lower byte.
                AND R0, R0, R2;
                BRz @TRAP_PUTSP_RETURN; // If it's 0 (NULL), we're done.

                OUT;                    // Otherwise, print it out.

                // Now the upper byte. To get it into the lower 8 bits of the word we
                // iteratively shift left, check the top bit, and append it to the
                // output which we also shift left.
                AND R0, R0, #0;         // R0 shall be the upper byte.
                ADD R3, R0, #8;         // Set R3 to 8: the number of iterations
                                        // we need to run to move the upper byte.

                ADD R2, R2, #0;         // Set the condition codes on R2 once.

                @TRAP_PUTSP_UPPER_BYTE_LOOP
                    BRzp @TRAP_PUTSP_CURRENT_MSB_LOW;
                    ADD R0, R0, #1;         // If the current MSB is set, append
                                            // a 1 to the output.

                    @TRAP_PUTSP_CURRENT_MSB_LOW
                    ADD R0, R0, R0;         // Shift the output left.

                    ADD R3, R3, #-1;        // Decrement the counter and break
                    BRz @TRAP_PUTSP_UPPER;  // from this loop if we're done.

                    ADD R2, R2, R2;         // Shift the input left and repeat.
                    BRnzp @TRAP_PUTSP_UPPER_BYTE_LOOP;

                @TRAP_PUTSP_UPPER
                    ADD R0, R0, #0;         // Once again, if it's 0 (NULL),
                    BRz @TRAP_PUTSP_RETURN; // we're done.

                    OUT;                    // Otherwise, print the character,
                    ADD R1, R1, #1;         // rinse,
                    BRnzp @TRAP_PUTSP_LOOP; // and repeat.

        @TRAP_PUTSP_RETURN
            LDR R3, R6, #0;          // Restore R0, R1, R2, and R3
            LDR R2, R6, #1;
            LDR R1, R6, #2;
            LDR R0, R6, #3;
            ADD R6, R6, #4;
            RTI;

        // HALT: Halts the machine!
        //
        // This routine lowers the MSB on the MCR to stop the machine (as it
        // should be done) but also does so in an infinite loop just in case the
        // simulator we're running on doesn't actually implement the MCR.
        @TRAP_HALT
            LEA R0, @TRAP_HALT_MSG;   // We're going down!
            PUTS;

            LDI R0, @MCR;             // Set the top bit of the MCR to 0.
            LD R1, @MASK_HI_BIT;      // The masking isn't strictly necessary
            AND R0, R0, R1;           // since (afaik) only the top bit of the
            STI R0, @MCR;             // MCR has functionality, but we'll be
                                      // good citizens.

            BRnzp @TRAP_HALT;         // If at first you don't succeed, try, try
                                      // again.

        // Triggered when an unregistered trap is called.
        //
        // Prints a message and halts the machine.
        @UNKNOWN_TRAP
            LEA R0, @UNKNOWN_TRAP_MSG;
            PUTS;
            HALT;

        // Some strings:
        @TRAP_IN_MSG // "\nInput a character> "
            .FILL #('\n' as Word);
            .FILL #('I' as Word);
            .FILL #('n' as Word);
            .FILL #('p' as Word);
            .FILL #('u' as Word);
            .FILL #('t' as Word);
            .FILL #(' ' as Word);
            .FILL #('a' as Word);
            .FILL #(' ' as Word);
            .FILL #('c' as Word);
            .FILL #('h' as Word);
            .FILL #('a' as Word);
            .FILL #('r' as Word);
            .FILL #('a' as Word);
            .FILL #('c' as Word);
            .FILL #('t' as Word);
            .FILL #('e' as Word);
            .FILL #('r' as Word);
            .FILL #('>' as Word);
            .FILL #(' ' as Word);
            .FILL #('\0' as Word);

        @TRAP_HALT_MSG // "\n\n--- Halting the LC-3 ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('H' as Word);
            .FILL #('a' as Word);
            .FILL #('l' as Word);
            .FILL #('t' as Word);
            .FILL #('i' as Word);
            .FILL #('n' as Word);
            .FILL #('g' as Word);
            .FILL #(' ' as Word);
            .FILL #('t' as Word);
            .FILL #('h' as Word);
            .FILL #('e' as Word);
            .FILL #(' ' as Word);
            .FILL #('L' as Word);
            .FILL #('C' as Word);
            .FILL #('-' as Word);
            .FILL #('3' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        @UNKNOWN_TRAP_MSG // "\n\n--- Undefined TRAP executed! ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('U' as Word);
            .FILL #('n' as Word);
            .FILL #('d' as Word);
            .FILL #('e' as Word);
            .FILL #('f' as Word);
            .FILL #('i' as Word);
            .FILL #('n' as Word);
            .FILL #('e' as Word);
            .FILL #('d' as Word);
            .FILL #(' ' as Word);
            .FILL #('T' as Word);
            .FILL #('R' as Word);
            .FILL #('A' as Word);
            .FILL #('P' as Word);
            .FILL #(' ' as Word);
            .FILL #('e' as Word);
            .FILL #('x' as Word);
            .FILL #('e' as Word);
            .FILL #('c' as Word);
            .FILL #('u' as Word);
            .FILL #('t' as Word);
            .FILL #('e' as Word);
            .FILL #('d' as Word);
            .FILL #('!' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        // Default entry for exceptions in the exception vector table.
        @DEFAULT_EXCEPTION_HANDLER
            LD R0, @DEFAULT_EX_MSG;
            PUTS;
            HALT;

        // Default entry for interrupts in the interrupt vector table;
        @DEFAULT_INT_HANDLER
            LD R0, @DEFAULT_INT_MSG;
            PUTS;
            HALT;

        // The rest of the strings (for offset reasons):
        @DEFAULT_EX_MSG // "\n\n--- Encountered an exception without a handler! ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('E' as Word);
            .FILL #('n' as Word);
            .FILL #('c' as Word);
            .FILL #('o' as Word);
            .FILL #('u' as Word);
            .FILL #('n' as Word);
            .FILL #('t' as Word);
            .FILL #('e' as Word);
            .FILL #('r' as Word);
            .FILL #('e' as Word);
            .FILL #('d' as Word);
            .FILL #(' ' as Word);
            .FILL #('a' as Word);
            .FILL #('n' as Word);
            .FILL #(' ' as Word);
            .FILL #('e' as Word);
            .FILL #('x' as Word);
            .FILL #('c' as Word);
            .FILL #('e' as Word);
            .FILL #('p' as Word);
            .FILL #('t' as Word);
            .FILL #('i' as Word);
            .FILL #('o' as Word);
            .FILL #('n' as Word);
            .FILL #(' ' as Word);
            .FILL #('w' as Word);
            .FILL #('i' as Word);
            .FILL #('t' as Word);
            .FILL #('h' as Word);
            .FILL #('o' as Word);
            .FILL #('u' as Word);
            .FILL #('t' as Word);
            .FILL #(' ' as Word);
            .FILL #('a' as Word);
            .FILL #(' ' as Word);
            .FILL #('h' as Word);
            .FILL #('a' as Word);
            .FILL #('n' as Word);
            .FILL #('d' as Word);
            .FILL #('l' as Word);
            .FILL #('e' as Word);
            .FILL #('r' as Word);
            .FILL #('!' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        @DEFAULT_INT_MSG // "\n\n--- Unhandled interrupt! ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('U' as Word);
            .FILL #('n' as Word);
            .FILL #('h' as Word);
            .FILL #('a' as Word);
            .FILL #('n' as Word);
            .FILL #('d' as Word);
            .FILL #('l' as Word);
            .FILL #('e' as Word);
            .FILL #('d' as Word);
            .FILL #(' ' as Word);
            .FILL #('i' as Word);
            .FILL #('n' as Word);
            .FILL #('t' as Word);
            .FILL #('e' as Word);
            .FILL #('r' as Word);
            .FILL #('r' as Word);
            .FILL #('u' as Word);
            .FILL #('p' as Word);
            .FILL #('t' as Word);
            .FILL #('!' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        // Checks if R0 is within range of 0 to R4
        // R0 = value to check
        // R4 = max value
        // -> cc = n if out of bounds
        //         p if within bounds
        // Does not modify R0
        // Destroys R1
        @CHECK_OUT_OF_BOUNDS
            ADD R0, R0, #0;                 // Check if R0 is negative
            BRn @OUT_OF_BOUNDS_RET;
            NOT R4, R4;                     // Negate R4
            ADD R4, R4, #1;
            ADD R4, R0, R4;                 // Check if R0 is less than R4
            BRp @OUT_OF_BOUNDS;
            ADD R0, R0, #0;                 // If not, set cc to p
            BR @OUT_OF_BOUNDS_RET;
        @OUT_OF_BOUNDS
            NOT R4, R0;                     // Set cc to n
        @OUT_OF_BOUNDS_RET
            RET;

        // Enables GPIO pin
        // R0 = GPIO pin to enable
        // R1 = mode to set
        @SET_GPIO_MODE
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of GPIO pins
            ADD R4, R4, #lc3_traits::peripherals::gpio::GpioPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_SET_GPIO_MODE;

            LD R4, @OS_GPIO_BASE_ADDR;      // Load GPIO base address into R2
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains control address of pin number in R0
            STR R1, R4, #0;                 // Write GPIO mode to control register
        @SKIP_SET_GPIO_MODE
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RET;

        // Sets GPIO pin to input mode
        // R0 = GPIO pin to set
        @TRAP_SET_GPIO_INPUT
            ADD R6, R6, #-2;                // Save R1, R7 on stack
            STR R1, R6, #1;
            STR R7, R6, #0;

            AND R1, R1, #0;                 // Set R1 to 2 (Input)
            ADD R1, R1, #2;
            JSR @SET_GPIO_MODE;
            LDR R7, R6, #0;                 // Restore R1, R7
            LDR R1, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Sets GPIO pin to output mode
        // R0 = GPIO pin to set
        @TRAP_SET_GPIO_OUTPUT
            ADD R6, R6, #-2;                // Save R1, R7 on stack
            STR R1, R6, #1;
            STR R7, R6, #0;

            AND R1, R1, #0;                 // Set R1 to 1 (Output)
            ADD R1, R1, #1;
            JSR @SET_GPIO_MODE;
            LDR R7, R6, #0;                 // Restore R1, R7
            LDR R1, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Sets GPIO pin to interrupt mode and sets ISR address in IVT
        // R0 = GPIO pin to set
        // R1 = Address of interrupt service routine
        @TRAP_SET_GPIO_INTERRUPT
            ADD R6, R6, #-3;                // Save R1, R4, R7 on stack
            STR R1, R6, #2;
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of GPIO pins
            ADD R4, R4, #lc3_traits::peripherals::gpio::GpioPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_SET_GPIO_INTERRUPT;

            LD R4, @OS_GPIO_BASE_INTVEC;    // Load GPIO base interrupt vector address
            ADD R4, R4, R0;                 // R4 contains address of pin in R0
            STR R1, R4, #0;                 // Load service routine address into vector table

            LD R4, @OS_GPIO_BASE_ADDR;      // Load GPIO base address into R4
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains control address of pin number in R0
            AND R1, R1, #0;                 // Set R1 to 3 (Interrupt)
            ADD R1, R1, #3;
            STR R1, R4, #0;                 // Write GPIO mode to control register
        @SKIP_SET_GPIO_INTERRUPT
            LDR R7, R6, #0;                 // Restore R1, R4, R7
            LDR R4, R6, #1;
            LDR R1, R6, #2;
            ADD R6, R6, #3;
            RTI;

        // Sets GPIO pin to disabled
        // R0 = GPIO pin to set
        @TRAP_SET_GPIO_DISABLED
            ADD R6, R6, #-2;                // Save R1, R7 on stack
            STR R1, R6, #1;
            STR R7, R6, #0;

            AND R1, R1, #0;                 // Set R1 to 0 (Disabled)
            JSR @SET_GPIO_MODE;
            LDR R7, R6, #0;                 // Restore R1, R7
            LDR R1, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns mode of GPIO pin
        // R0 = GPIO pin to read from
        // -> R0 = mode of GPIO pin
        @TRAP_READ_GPIO_MODE
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of GPIO pins
            ADD R4, R4, #lc3_traits::peripherals::gpio::GpioPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_GPIO_MODE;

            LD R4, @OS_GPIO_BASE_ADDR;      // Load GPIO base address into R2
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R3 contains data address of pin number in R0
            LDR R0, R4, #0;                 // Reads mode from pin into R0
        @SKIP_READ_GPIO_MODE
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Writes data to GPIO pin
        // R0 = GPIO pin to write to
        // R1 = data to write
        @TRAP_WRITE_GPIO_DATA
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of GPIO pins
            ADD R4, R4, #lc3_traits::peripherals::gpio::GpioPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_WRITE_GPIO_DATA;

            LD R4, @OS_GPIO_BASE_ADDR;      // Load GPIO base address into R2
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // and adding 1
            ADD R4, R4, #1;                 // R4 contains data address of pin number in R0
            STR R1, R4, #0;                 // Writes data from R1 to pin in R0
        @SKIP_WRITE_GPIO_DATA
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns data from GPIO pin
        // R0 = GPIO pin to read from
        // -> R0 = data from GPIO pin
        @TRAP_READ_GPIO_DATA
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of GPIO pins
            ADD R4, R4, #lc3_traits::peripherals::gpio::GpioPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_GPIO_DATA;

            LD R4, @OS_GPIO_BASE_ADDR;      // Load GPIO base address into R1
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // and adding 1
            ADD R4, R4, #1;                 // R3 contains data address of pin number in R0
            LDR R0, R4, #0;                 // Reads data from pin into R0
        @SKIP_READ_GPIO_DATA
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Sets mode of ADC pin
        // R0 = ADC pin to set mode of
        // R1 = mode to set
        @SET_ADC_MODE
            ADD R6, R6, #-3;                // Save R1, R4, R7 on stack
            STR R1, R6, #2;
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of ADC pins
            ADD R4, R4, #lc3_traits::peripherals::adc::AdcPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_SET_ADC_MODE;

            LD R4, @OS_ADC_BASE_ADDR;       // Load ADC base address into R2
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains control address of pin number in R0
            STR R1, R4, #0;                 // Writes ADC mode to control register
        @SKIP_SET_ADC_MODE
            LDR R7, R6, #0;                 // Restore R1, R4, R7
            LDR R4, R6, #1;
            LDR R1, R6, #2;
            ADD R6, R6, #3;
            RET;

        // Sets mode of ADC pin to Enabled
        // R0 = ADC pin to enable
        @TRAP_SET_ADC_ENABLE
            ADD R6, R6, #-2;                // Save R1, R7 on stack
            STR R1, R6, #1;
            STR R7, R6, #0;

            AND R1, R1, #0;                 // Sets mode to 1, to enable ADC
            ADD R1, R1, #1;
            JSR @SET_ADC_MODE;
            LDR R7, R6, #0;                 // Restore R1, R7
            LDR R1, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Sets mode of ADC pin to Disabled
        // R0 = ADC pin to disable
        @TRAP_SET_ADC_DISABLE
            ADD R6, R6, #-2;                // Save R1, R7 on stack
            STR R1, R6, #1;
            STR R7, R6, #0;

            AND R1, R1, #0;                 // Sets mode to 0, which is mode to disable ADC
            JSR @SET_ADC_MODE;
            LDR R7, R6, #0;                 // Restore R1, R7
            LDR R1, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns mode of ADC pin
        // R0 = ADC pin to read from
        // -> R0 = mode of ADC pin
        @TRAP_READ_ADC_MODE
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of ADC pins
            ADD R4, R4, #lc3_traits::peripherals::adc::AdcPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_ADC_MODE;

            LD R4, @OS_ADC_BASE_ADDR;       // Load ADC base address into R2
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R3 contains control address of pin number in R0
            LDR R0, R4, #0;                 // Reads mode from pin into R0
        @SKIP_READ_ADC_MODE
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns data from ADC pin
        // R0 = ADC pin to read from
        // -> R0 = data from ADC pin
        @TRAP_READ_ADC_DATA
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of ADC pins
            ADD R4, R4, #lc3_traits::peripherals::adc::AdcPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_ADC_DATA;

            LD R4, @OS_ADC_BASE_ADDR;       // Load ADC base address into R1
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // and add 1
            ADD R4, R4, #1;                 // R3 contains data address of pin number in R0
            LDR R0, R4, #0;                 // Reads data from pin in R0
        @SKIP_READ_ADC_DATA
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // More constants
        @OS_GPIO_BASE_ADDR .FILL #G0CR_ADDR;
        @OS_ADC_BASE_ADDR .FILL #A0CR_ADDR;
        @OS_CLOCK_BASE_ADDR .FILL #CLKR_ADDR;
        @OS_TIMER_BASE_ADDR .FILL #T0CR_ADDR;
        @OS_PWM_BASE_ADDR .FILL #P0CR_ADDR;

        @OS_GPIO_BASE_INTVEC .FILL #GPIO_BASE_INT_VEC;
        @OS_TIMER_BASE_INTVEC .FILL #TIMER_BASE_INT_VEC;

        // PWM set
        // R0 = PWM pin to set
        // R1 = period to set
        // R2 = duty cycle to set
        @TRAP_SET_PWM
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of PWM pins
            ADD R4, R4, #lc3_traits::peripherals::pwm::PwmPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_SET_PWM;

            LD R4, @OS_PWM_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains address of period control register
            STR R1, R4, #0;                 // Write period to PWM
            ADD R4, R4, #1;                 // R4 contains address of duty cycle register
            STR R2, R4, #0;                 // Write duty cycle to PWM
        @SKIP_SET_PWM
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // PWM disable
        // R0 = PWM to disable
        @TRAP_DISABLE_PWM
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of PWM pins
            ADD R4, R4, #lc3_traits::peripherals::pwm::PwmPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_DISABLE_PWM;

            LD R4, @OS_PWM_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains address of period control register
            AND R7, R7, #0;
            STR R7, R4, #0;                 // Disable PWM (period = 0)
        @SKIP_DISABLE_PWM
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns period of PWM pin
        // R0 = PWM pin to read from
        // -> R0 = mode of PWM pin
        @TRAP_READ_PWM_PERIOD
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of PWM pins
            ADD R4, R4, #lc3_traits::peripherals::pwm::PwmPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_PWM_PERIOD;

            LD R4, @OS_PWM_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R3 contains control address of pin number in R0
            LDR R0, R4, #0;                 // Reads mode from pin into R0

        @SKIP_READ_PWM_PERIOD
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns data from PWM pin
        // R0 = PWM pin to read from
        // -> R0 = data from PWM pin
        @TRAP_READ_PWM_DUTY_CYCLE
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of PWM pins
            ADD R4, R4, #lc3_traits::peripherals::pwm::PwmPin::NUM_PINS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_PWM_DUTY_CYCLE;

            LD R4, @OS_PWM_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // and adding 1
            ADD R4, R4, #1;                 // R3 contains data address of pin number in R0
            LDR R0, R4, #0;                 // Reads data from pin into R0
        @SKIP_READ_PWM_DUTY_CYCLE
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Timer Pin Set
        // R0= Timer Pin to set mode of
        // R1= mode to be set
        @SET_TIMER_MODE
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of timers
            ADD R4, R4, #lc3_traits::peripherals::timers::TimerId::NUM_TIMERS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_SET_TIMER_MODE;

            LD R4, @OS_TIMER_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains address of pin number in R0
            STR R1, R4, #0;
        @SKIP_SET_TIMER_MODE
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RET;

        // Writes period to TIMER pin
        // R0 = TIMER pin to write to
        // R1 = data to write
        @WRITE_TIMER_PERIOD
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of timers
            ADD R4, R4, #lc3_traits::peripherals::timers::TimerId::NUM_TIMERS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_WRITE_TIMER_PERIOD;

            LD R4, @OS_TIMER_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // and adding 1
            ADD R4, R4, #1;                 // R4 contains data address of pin number in R0
            STR R1, R4, #0;                 // Writes data from R1 to pin in R0
        @SKIP_WRITE_TIMER_PERIOD
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RET;

        // Sets timer to SingleShot mode with period
        // R0 = TIMER pin to write to
        // R1 = period to be set
        // R2 = address of interrupt service routine
        @TRAP_SET_TIMER_SINGLESHOT
            ADD R6, R6, #-2;                // Save R7, R1 on stack
            STR R7, R6, #1;
            STR R1, R6, #0;

            AND R1, R1, #0;
            JSR @SET_TIMER_MODE;

            LDR R1, R6, #0;                 // Restore R1
            ADD R6, R6, #1;
            JSR @WRITE_TIMER_PERIOD;

            LD R1, @OS_TIMER_BASE_INTVEC;
            ADD R1, R1, R0;
            STR R2, R1, #0;

            LDR R7, R6, #0;                 // Restore R7
            ADD R6, R6, #1;
            RTI;

        // Sets timer to Repeated mode with period
        // R0 = Timer pin to write to
        // R1 = period to be set
        // R2 = address of interrupt service routine
        @TRAP_SET_TIMER_REPEAT
            ADD R6, R6, #-2;                // Save R7, R1 on stack
            STR R7, R6, #1;
            STR R1, R6, #0;

            AND R1, R1, #0;
            ADD R1, R1, #1;
            JSR @SET_TIMER_MODE;

            LDR R1, R6, #0;                 // Restore R1
            ADD R6, R6, #1;
            JSR @WRITE_TIMER_PERIOD;

            LD R1, @OS_TIMER_BASE_INTVEC;
            ADD R1, R1, R0;
            STR R2, R1, #0;

            LDR R7, R6, #0;                 // Restore R7
            ADD R6, R6, #1;
            RTI;

        // Sets timer's state to Disabled
        // R0 = Timer pin to disable
        @TRAP_SET_TIMER_DISABLE
            ADD R6, R6, #-2;                // Save R1, R7 on stack
            STR R1, R6, #1;
            STR R7, R6, #0;

            AND R1, R1, #0;
            JSR @WRITE_TIMER_PERIOD;
            LDR R7, R6, #0;                 // Restore R1, R7
            LDR R1, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns mode of Timer pin
        // R0 = Timer pin to read from
        // -> R0 = mode of Timer pin
        @TRAP_READ_TIMER_MODE
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of timers
            ADD R4, R4, #lc3_traits::peripherals::timers::TimerId::NUM_TIMERS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_TIMER_MODE;

            LD R4, @OS_TIMER_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // R4 contains control address of pin number in R0
            LDR R0, R4, #0;                 // Reads mode from pin into R0
        @SKIP_READ_TIMER_MODE
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Reads and returns data from PWM pin
        // R0 = TIMER pin to read from
        // -> R0 = data from TIMER pin
        @TRAP_READ_TIMER_PERIOD
            ADD R6, R6, #-2;                // Save R4, R7 on stack
            STR R4, R6, #1;
            STR R7, R6, #0;

            AND R4, R4, #0;                 // Set R4 to # of timers
            ADD R4, R4, #lc3_traits::peripherals::timers::TimerId::NUM_TIMERS as i16;
            JSR @CHECK_OUT_OF_BOUNDS;
            BRn @SKIP_READ_TIMER_PERIOD;

            LD R4, @OS_TIMER_BASE_ADDR;
            ADD R4, R4, R0;                 // Calculate pin address offset by doubling pin number
            ADD R4, R4, R0;                 // and adding 1
            ADD R4, R4, #1;                 // R4 contains data address of pin number in R0
            LDR R0, R4, #0;                 // Reads data from pin into R0
        @SKIP_READ_TIMER_PERIOD
            LDR R7, R6, #0;                 // Restore R4, R7
            LDR R4, R6, #1;
            ADD R6, R6, #2;
            RTI;

        // Sets clock
        // R0 = data to set
        @TRAP_SET_CLOCK
            ADD R6, R6, #-1;                // Save R1 on stack
            STR R1, R6, #0;

            LD R1, @OS_CLOCK_BASE_ADDR;     // Load clock base address into R1
            STR R0, R1, #0;                 // Write data in R0 to clock

            LDR R1, R6, #0;                 // Restore R1
            ADD R6, R6, #1;
            RTI;

        // Reads clock
        // -> R0 = data read from clock
        @TRAP_READ_CLOCK
            LD R0, @OS_CLOCK_BASE_ADDR;     // Load clock base address into R1
            LDR R0, R0, #0;                 // Read data from clock
            RTI;

        //// Exception Handlers ////

        // Triggered when an RTI is called when in user mode.
        // Halts the machine.
        @PRIVILEGE_MODE_EX_HANDLER
            LEA R0, @PRIVILEGE_MODE_EX_MSG;
            PUTS;
            HALT;

        // Triggered when the illegal opcode (0b1101) is encountered.
        // Halts the machine.
        @ILLEGAL_OPCODE_EX_HANDLER
            LEA R0, @ILLEGAL_OPCODE_EX_MSG;
            PUTS;
            HALT;

        // Triggered when access control violations occur.
        //
        // TODO: not sure what will happen when this is told _not_ to error...
        @ACV_EX_HANDLER
            ADD R6, R6, #-1;
            STR R0, R6, #0; // Save R0;

            LEA R0, @ACV_EX_MSG; // Print the error message no matter what.
            PUTS;

            LDI R0, @ERROR_ON_ACV_SETTING_ADDR_PTR; // Check if we're supposed to actually
                                                    // error on ACVs.
            BRz @ACV_EX_HANDLER_EXIT;               // If we're not, just return.

            HALT;                // Otherwise, halt.

            @ACV_EX_HANDLER_EXIT // Restore R0 and return.
                LDR R0, R6, #0;
                ADD R6, R6, #1;
                RTI;

        // More constants
        @ERROR_ON_ACV_SETTING_ADDR_PTR .FILL #ERROR_ON_ACV_SETTING_ADDR;

        // More strings
        @PRIVILEGE_MODE_EX_MSG // "\n\n--- Privilege mode violation (RTI in user mode)! ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('P' as Word);
            .FILL #('r' as Word);
            .FILL #('i' as Word);
            .FILL #('v' as Word);
            .FILL #('i' as Word);
            .FILL #('l' as Word);
            .FILL #('e' as Word);
            .FILL #('g' as Word);
            .FILL #('e' as Word);
            .FILL #(' ' as Word);
            .FILL #('m' as Word);
            .FILL #('o' as Word);
            .FILL #('d' as Word);
            .FILL #('e' as Word);
            .FILL #(' ' as Word);
            .FILL #('v' as Word);
            .FILL #('i' as Word);
            .FILL #('o' as Word);
            .FILL #('l' as Word);
            .FILL #('a' as Word);
            .FILL #('t' as Word);
            .FILL #('i' as Word);
            .FILL #('o' as Word);
            .FILL #('n' as Word);
            .FILL #(' ' as Word);
            .FILL #('(' as Word);
            .FILL #('R' as Word);
            .FILL #('T' as Word);
            .FILL #('I' as Word);
            .FILL #(' ' as Word);
            .FILL #('i' as Word);
            .FILL #('n' as Word);
            .FILL #(' ' as Word);
            .FILL #('u' as Word);
            .FILL #('s' as Word);
            .FILL #('e' as Word);
            .FILL #('r' as Word);
            .FILL #(' ' as Word);
            .FILL #('m' as Word);
            .FILL #('o' as Word);
            .FILL #('d' as Word);
            .FILL #('e' as Word);
            .FILL #(')' as Word);
            .FILL #('!' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        @ILLEGAL_OPCODE_EX_MSG // "\n\n--- Illegal opcode (0b1101)! ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('I' as Word);
            .FILL #('l' as Word);
            .FILL #('l' as Word);
            .FILL #('e' as Word);
            .FILL #('g' as Word);
            .FILL #('a' as Word);
            .FILL #('l' as Word);
            .FILL #(' ' as Word);
            .FILL #('o' as Word);
            .FILL #('p' as Word);
            .FILL #('c' as Word);
            .FILL #('o' as Word);
            .FILL #('d' as Word);
            .FILL #('e' as Word);
            .FILL #(' ' as Word);
            .FILL #('(' as Word);
            .FILL #('0' as Word);
            .FILL #('b' as Word);
            .FILL #('1' as Word);
            .FILL #('1' as Word);
            .FILL #('0' as Word);
            .FILL #('1' as Word);
            .FILL #(')' as Word);
            .FILL #('!' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        @ACV_EX_MSG // "\n\n--- Access control violation! ---\n\n"
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #(' ' as Word);
            .FILL #('A' as Word);
            .FILL #('c' as Word);
            .FILL #('c' as Word);
            .FILL #('e' as Word);
            .FILL #('s' as Word);
            .FILL #('s' as Word);
            .FILL #(' ' as Word);
            .FILL #('c' as Word);
            .FILL #('o' as Word);
            .FILL #('n' as Word);
            .FILL #('t' as Word);
            .FILL #('r' as Word);
            .FILL #('o' as Word);
            .FILL #('l' as Word);
            .FILL #(' ' as Word);
            .FILL #('v' as Word);
            .FILL #('i' as Word);
            .FILL #('o' as Word);
            .FILL #('l' as Word);
            .FILL #('a' as Word);
            .FILL #('t' as Word);
            .FILL #('i' as Word);
            .FILL #('o' as Word);
            .FILL #('n' as Word);
            .FILL #('!' as Word);
            .FILL #(' ' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('-' as Word);
            .FILL #('\n' as Word);
            .FILL #('\n' as Word);
            .FILL #('\0' as Word);

        //// Configuration 'variables' ////
        // (binaries can override these)

        .ORIG #USER_PROG_START_ADDR;
        .FILL #lc3_isa::USER_PROGRAM_START_ADDR;

        .ORIG #ERROR_ON_ACV_SETTING_ADDR;
        .FILL #0x1; // 0 == disabled, non-zero == enabled

        .ORIG #OS_STARTING_SP_ADDR;
        .FILL #OS_DEFAULT_STARTING_SP;
    };

    AssembledProgram::new(os)
}]}
