// no std libs here lol. but C must stay. (actually, this is because of rusts's rules, breaking my code. whatever.)

// this will memcpy it over. so nice. 
typedef void (*push_jmpbuf_t)(void *);
extern int setjmp(void* buf);
extern __attribute__((noreturn)) void longjmp(void* buf, int code);
void changecontext(push_jmpbuf_t fcn, void* ref_to_a_buffer) {
    if (setjmp(ref_to_a_buffer) == 1) {
        return;
    }
    fcn(ref_to_a_buffer);
    longjmp(ref_to_a_buffer, 1);
}