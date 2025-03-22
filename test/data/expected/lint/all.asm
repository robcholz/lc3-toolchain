; Test file for LC-3 assembly parser coverage
; This file includes examples of all syntax elements defined in the grammar

.ORIG x3000    ; Program start directive with hex address

; Labels (with and without colons)
START
LOOP
SIMPLE_LABEL
LABEL_WITH_UNDERSCORE
LABEL123
    NOP

; Testing all instructions with various operand combinations
; Arithmetic and Logical Instructions
ADD R1, R2, R3          ; Register mode
ADD R0, R1 #5  ,        ; Immediate decimal mode
ADD R3 R4 x10  ,       ; Immediate hex mode

AND R1, R2, R3          ; Register mode
AND R0, R1, #-15        ; Immediate with negative value
AND R3, R4, xF          ; Immediate hex mode

NOT R1, R2              ; Bit-wise NOT

; Load and Store Instructions
LD  R0, DATAPTR         ; Load from memory
LDI R1, INDIRECTPTR     ; Load indirect
LDR R2, R3, #4          ; Load register offset
LEA R4, LOOP            ; Load effective address

ST  R0, DATAPTR         ; Store to memory
STI R1, INDIRECTPTR     ; Store indirect
STR R2, R3, #4          ; Store register offset

; Branching Instructions
BR  SOMEWHERE           ; Unconditional branch
BRN NEGATIVE            ; Branch if negative
BRZ ZERO                ; Branch if zero
BRP POSITIVE            ; Branch if positive
BRNZ NEGORZER0          ; Branch if negative or zero
BRNP NEGPOS             ; Branch if negative or positive
BRZP ZERPOS             ; Branch if zero or positive
BRNZP ALWAYS            ; Branch always (all conditions)

JMP R7                  ; Jump to address in register
JSR SUBROUTINE          ; Jump to subroutine
JSRR R6                 ; Jump to subroutine in register

; Control Instructions
NOP                     ; No operation
RET                     ; Return from subroutine
HALT                    ; Halt execution

; Input/Output Instructions
PUTS                    ; Output string
GETC                    ; Get character
OUT                     ; Output character
IN                      ; Input character with prompt

; TRAP Instructions
TRAP x23                ; System trap with hex value

; Testing directives
.FILL #42               ; Fill with decimal value
.FILL x2A               ; Fill with hex value
.BLKW #10               ; Block with 10 words
.STRINGZ "This is a test string"  ; String definition

; Testing comments
; This is a comment line
ADD R0, R0, #1 ; This is an inline comment

; Labels that might be confused with instructions
ADD_DATA              ; Should be recognized as a label, not ADD instruction
BR_TARGET             ; Should be recognized as a label, not BR instruction
AND_MASK              ; Should be recognized as a label, not AND instruction
NOT_VALUE             ; Should be recognized as a label, not NOT instruction
LD_OFFSET             ; Should be recognized as a label, not LD instruction
ST_LOCATION           ; Should be recognized as a label, not ST instruction

; Testing immediate values
ADD R0, R0, #0         ; Decimal zero
ADD R0, R0, #-1        ; Negative decimal
ADD R0, R0, #+100      ; Positive decimal with plus sign
ADD R0, R0, x0         ; Hex zero
ADD R0, R0, xFF        ; Hex FF

; Testing all BR variants
BR LABEL1              ; Unconditional
BRN LABEL2             ; Negative
BRZ LABEL3             ; Zero
BRP LABEL4             ; Positive
BRNZ LABEL5            ; Negative or zero
BRNP LABEL6            ; Negative or positive
BRZP LABEL7            ; Zero or positive
BRNZP LABEL8           ; Always

; Data area with various label names
LABEL1 .FILL x1234
LABEL2 .FILL #-5000
SOMEWHERE .FILL x2000
NEGATIVE .FILL #-1
ZERO .FILL #0
POSITIVE .FILL #1
NEGORZER0 .FILL x8000
NEGPOS .FILL #-100
ZERPOS .FILL #0
ALWAYS .FILL #1
DATAPTR .FILL xF0F0
INDIRECTPTR .FILL x8080
SUBROUTINE .FILL x4000

.END                    ; End of program