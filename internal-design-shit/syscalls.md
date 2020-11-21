Here is what i want:
0. exit(exit_code)
1. bindbuffer(buffer_addr, buffer_len)
2. getbufferlen() -> buffer_len
3. readbuffer(buffer_addr) -> buffer_len
4. swapbuffers()
5. send(target) [buf1 = postcard data, buf2 = auxilary data] -> [buf1 = response data, buf2 = auxilary data]
6. listen(name) -> 0
7. accept(name) -> qid [buf1 = postcard data, buf2 = auxilary data] (qid & 1 = is root, qid & 2)
8. exec() [buf1 = program, buf2 = argv blob] -> pid
9. respond() [buf1 = response data, buf2 = auxilary data]
10. klog(str) 
11. sbrk see da manseite
On start buf1 has the argv blob.
exec: from target fs, use read_for_exec(as_pcrd(target_name)) resulting in buf1 being Ok(()) and buf2 being the target ELF.
