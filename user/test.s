[BITS 64]
%define sys_exit 0
%define sys_bindbuffer 1
%define sys_getbufferlen 2
%define sys_readbuffer 3
%define sys_swapbuffers 4
%define sys_send 5
%define sys_listen 6
%define sys_accept 7
%define sys_exec 8
%define sys_respond 9
%define sys_klog 10
%macro do_syscall 3
    push rcx
    push r11
    mov rdi, %1
    mov rsi, %2
    mov r8, %3
    syscall
    pop r11
    pop rcx
%endmacro
default rel
section .text
user:
    mov rsp, stack_bottom
    lea rax, [hello_world]
    do_syscall sys_klog, rax, 0
    jmp $
section .data
hello_world:
    db "Hello, userland world!", 0
align 8
stack_top:
    resb 4096
stack_bottom: