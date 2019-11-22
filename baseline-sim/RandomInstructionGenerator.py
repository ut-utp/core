import random


instr_set = ["ADD",
            "AND",
            "BR",
            "JMP",
            "JSR",
            "JSRR",
            "LD",
            "LDI",
            "LDR",
            "LEA",
            "NOT",
            "RET",
            "RTI",
            "ST",
            "STR"
            ]

reg_set = ["R0",
           "R1",
           "R2",
           "R3",
           "R4",
           "R5",
           "R6",
           "R7"
           ]

restricted_reg_set= ["R0",
                     "R1",
                     "R2",
                     "R3",
                     "R4",
                     "R5",
                     "R6"
                     ]
    
def getrandom():

# =============================================================================
#     for items in instr_set:
#         print(items)
#         
# =============================================================================
    instruction = random.choice(instr_set)    
#    print(instruction)
    
    randreg = random.choice(reg_set)
#    print(randreg)
    
    randreg2 = random.choice(reg_set)
#    print(randreg2)
    
    randreg3 = random.choice(reg_set)
#    print(randreg3)
    
    randnum = random.randrange(100)
#    print(randnum)
    
    randoff = str(random.randrange(-15,15))
#    print(randnum2)
    
    fiftyfifty= random.randrange(0,1)
#    print(fiftyfifty)
    
    label = 'LOOP' #defime any label here
    
    return {
            "ADD": instruction +' '+randreg2+','+randreg3+','+randreg ,
            "AND": instruction +' '+randreg2+','+randreg3+','+randreg ,
            "BR": instruction +' '+ label ,
            "JMP": instruction +' '+randreg2,
            "JSR": instruction +' '+ label ,
            "JSRR": instruction +' '+randreg2,
            "LD": instruction +' '+randreg2+','+label ,
            "LDI": instruction +' '+randreg2+','+label,
            "LDR": instruction +' '+randreg2+','+randoff,
            "LEA": instruction +' '+randreg2+','+label ,
            "NOT": instruction +' '+randreg2+','+randreg3,
            "RET": instruction,
            "RTI": instruction,
            "ST": instruction +' '+randreg2+','+label,
            "STR": instruction +' '+randreg2+','+randreg3+','+randoff 
    }[instruction]
    
# =============================================================================
#     if (instruction == "ADD"):
#         result = "
#     
#     
# =============================================================================
                
def main():
    print(getrandom())

if __name__ == "__main__":
    main()