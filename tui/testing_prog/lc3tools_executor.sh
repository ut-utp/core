FILE1=$1
FILEOUT=$2
line=$(head -n 1 $FILE1)
tail -n +2 "$FILE1" > "$FILE1.asm"
../../../lc3tools/build/bin/assembler $FILE1.asm
(echo "regs"; echo "run ${line}"; echo "regs"; echo "mem 0 0xFF00"; echo "quit";) |../../../lc3tools/build/bin/simulator $FILE1.obj > $FILEOUT

