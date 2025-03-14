.ORIG x3000                       ;Program starts at memory location x3000

;----- Load and Store -----       
    LD R0, VALUE                  ;Load VALUE into R0 (PC-relative addressing)
    LDI R1, PTR                   ;Load indirect from address stored at PTR
    LDR R2, R0, #0                ;Load from R0 with an offset
    LEA R3, MESSAGE               ;Load effective address of MESSAGE into R3
    ST R0, VALUE                  ;Store R0 at VALUE (PC-relative)
    STI R1, PTR                   ;Store indirect using PTR
    STR R2, R0, #0                ;Store R2 at R0+offset 0

;----- Arithmetic and Logic ----- 
    ADD R4, R1, R2                ;R4 = R1 + R2
    ADD R4, R4, #5                ;R4 = R4 + 5 (immediate mode)
    AND R5, R1, R2                ;R5 = R1 AND R2
    AND R5, R5, #0                ;Clear R5 (bitwise AND with 0)
    NOT R6, R1                    ;R6 = bitwise NOT of R1

;----- Branching -----            
    BRz SKIP                      ;Branch to SKIP if zero flag is set
    BRp POSITIVE                  ;Branch if positive
    BRn NEGATIVE                  ;Branch if negative
    BRnzp CONTINUE                ;Unconditional branch

SKIP:
    NOP                           ;No operation (useful as a placeholder)

;----- Control -----              
    JMP R3                        ;Jump to address stored in R3
    JSR SUBROUTINE                ;Jump to subroutine
    JSR R                         ;Jump to subroutine using register

;----- Input/Output -----         
R4:
    GET                           ;Read a character from keyboard into R0
C:
    OUT                           ;Print character in R0
    PUT                           ;Print a null-terminated string from R0

S:
    IN                            ;Prompt and read a character

;----- TRAP Routines -----        
    TRAP x20                      ;GETC - Read a char
    TRAP x21                      ;OUT - Print a char
    TRAP x22                      ;PUTS - Print a string
    TRAP x23                      ;IN - Read a char with echo
    TRAP x25                      ;HALT - Stop execution

;----- Subroutines -----          
SUBROUTINE:
    ADD R7, R7, #-1               ;Example operation in subroutine
    RET                           ;Return from subroutine

POSITIVE:
    ADD R0, R0, #1                ;Example positive case
    BRnzp CONTINUE                ;Continue execution

NEGATIVE:
    ADD R0, R0, #-1               ;Example negative case
    BRnzp CONTINUE                ;Continue execution

CONTINUE:
    HALT                          ;End program

;----- Data Section -----         
VALUE:
   .FILL x1234                    ;Store a value in memory
PTR:
   .FILL x4000                    ;Address of a location in memory
MESSAGE:
   .STRINGZ "Hello, LC-3!"        
   .BLKW x10                      

.END                              ;End of program
