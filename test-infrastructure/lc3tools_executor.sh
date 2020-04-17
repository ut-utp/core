#!/usr/bin/env bash

set -e

FILE1="${1}"
LC3_BIN_DIR="${2:-""}"

asm() { "${LC3_BIN_DIR}/assembler" "${@}"; }
sim() { "${LC3_BIN_DIR}/simulator" "${@}"; }

num_insns=$(head -n 1 "$FILE1")
tail -n +2 "$FILE1" > "$FILE1.asm"

asm "${FILE1}.asm"

(cat <<-EOF
	run "${num_insns}"
	regs
	mem 0 0xFDFF
	regs
	quit
EOF
) | sim "${FILE1}.obj"

rm -f "${FILE1}.obj" "${FILE1}.asm" "${FILE1}" &> /dev/null
