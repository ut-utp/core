#!/usr/bin/env bash

set -e

FILE1="$1"
OUT_FILE="lc3tools_output_${2}.txt"
LC3_BIN_DIR="${3:-""}"

asm() { "${LC3_BIN_DIR}/assembler" "${@}"; }
sim() { "${LC3_BIN_DIR}/simulator" "${@}"; }

line=$(head -n 1 "$FILE1")
tail -n +2 "$FILE1" > "$FILE1.asm"

asm "${FILE1}.asm"

(cat <<-EOF
	run "${line}"
	regs
	mem 0 0xFDFF
	regs
	quit
EOF
) | sim "${FILE1}.obj" > "${OUT_FILE}"

rm "${FILE1}.obj"
rm "${FILE1}.asm"
rm "${FILE1}"
