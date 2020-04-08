FILE1=$1
line=$(head -n 1 $FILE1)
tail -n +2 "$FILE1" > "$FILE1.asm"
../../../lc3tools/build/bin/assembler $FILE1.asm
name=lc3tools_output_$2.txt
(echo "run ${line}"; echo "regs"; echo "mem 0 0xFDFF"; echo "regs"; echo "quit";) | ../../../lc3tools/build/bin/simulator $FILE1.obj > $name

rm "$FILE1.obj"
rm "$FILE1.asm"
rm "$FILE1"

