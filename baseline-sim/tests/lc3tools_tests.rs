extern crate lc3_test_infrastructure as lti;

// The bash script will not work on Windows.
#[cfg(all(test, target_family = "unix"))]
mod lc3tools {
    use lti::lc3_sequence;

    lc3_sequence!{
        add_one,
        insns: [
            { ADD R0, R0, #1 },
            { ADD R1, R1, #1 },
            { ADD R2, R2, #1 },
            { ADD R3, R3, #1 },
            { ADD R4, R4, #1 },
            { ADD R5, R5, #1 },
            { ADD R6, R6, #1 },
            { ADD R7, R7, #1 },
        ],
    }

    lc3_sequence!{
        set_memory,
        insns: [
            { ADD R0, R0, #1 },
            { ST R0, #5 },
            { LD R1, #4},
        ],
    }

    lc3_sequence!{
        add_and_set,
        insns: [
            { ADD R0, R0, #1 },
            { AND R0, R1, R0 },
            { ADD R2, R2, #1 },
            { ADD R0, R2, R2 },
            { AND R0, R0, R2 },
            { ADD R5, R5, #1 },
            { LD R5, #10 },
            { ADD R7, R7, #1 },
            { ST R0, #5 },
            { LD R1, #4},
        ],
    }
}
