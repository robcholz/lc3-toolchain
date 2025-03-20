;LC-3 Assembly Test File                        
;This program adds two numbers stored in memory 
.ORIG x3000                                     




    LD R1, NUM1                                 ;Load first number into R1
    LD, R2, NUM2                                 ;Load second number into R2
    ADD R3, R1, R2                              ;Add R1 and R2, store in R3
    ST R3, RESULT                               ;Store result in memory
    HALT                                        ;Stop execution

NUM1
   .FILL x0005                                  ;First number (5)
NUM2:
   .FILL x0003                                  ;Second number (3)
RESULT
   .BLKW 1                                      ;Reserve space for result

.END                                            
