;LC-3 Assembly Test File                        
;This program adds two numbers stored in memory 
.ORIG x3000                                     

    LD R1, NUM1                                 ;Load first number into R1
    LD R2, NUM2                                 ;Load second number into R2
    ADD R3, R1, R2                              ;Add R1 and R2, store in R3
    ST R3, RESULT                               ;Store result in memory
    HALT                                        ;Stop execution

input:
   .FILL x0005                                  ;First number (5)
in_asd:
   .FILL x0003                                  ;Second number (3)
br_:
   .FILL x0003                                  ;Second number (3)
brnzp_:
   .FILL x0003                                  ;Second number (3)
br1:
   .FILL x0003                                  ;Second number (3)
l:
   .BLKW 1                                      ;Reserve space for result
b1:
   .BLKW 1                                      ;Reserve space for result

.END                                            
