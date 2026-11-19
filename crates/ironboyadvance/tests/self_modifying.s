.arm
.text

start:
    mov r5, #0x03000000
    mov r4, r5
    add r4, r4, #0x100

    adr r1, function_one
    ldr r3, [r1], #4
    str r3, [r5]
    ldr r3, [r1]
    str r3, [r5, #4]
    mov r1, r5
    mov lr, pc
    bx r1
    str r0, [r4]

    adr r1, function_two
    ldr r3, [r1], #4
    str r3, [r5]
    ldr r3, [r1]
    str r3, [r5, #4]
    mov r1, r5
    mov lr, pc
    bx r1
    str r0, [r4, #4]

    adr r1, function_three
    ldmia r1, {r6, r7, r8, r9, r10}
    stmia r5, {r6, r7, r8, r9, r10}
    add r1, r5, #12
    adr r2, replacement
    ldr r2, [r2]
    mov lr, pc
    bx r5
    str r0, [r4, #8]

spin:
    b spin

function_one:
    mov r0, #1
    bx lr

function_two:
    mov r0, #2
    bx lr

function_three:
    str r2, [r1]
    mov r0, r0
    mov r0, r0
    mov r0, #1
    bx lr

replacement:
    mov r0, #2
