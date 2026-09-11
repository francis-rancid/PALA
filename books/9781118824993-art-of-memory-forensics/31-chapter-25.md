# Chapter 25  
 Userland Rootkits

When adversaries design rootkits, one of the first decisions that they must make is whether the rootkit will operate in userland (process memory) or kernel mode. Kernel mode rootkits offer the most power, such as Direct Kernel Object Manipulation (DKOM) capabilities, direct interaction with hardware devices, and the capability to perform certain privileged operations. However, many common rootkit tasks such as hiding processes, logging keystrokes, and snooping on network activity can all be accomplished in userland. Furthermore, userland rootkits are more portable, whereas the kernel mode counterparts are difficult to maintain due to the rapidly changing Linux kernel. Another issue is that a number of system administration tools and Host Intrusion Prevention Systems (HIPS) perform kernel mode rootkit detection. On the other hand, userland rootkit detection has so far received only minor attention, which gives the attacker greater freedom over what techniques can be used and might prolong their access to the target system.

In previous chapters, you learned how to classify userland process activity based on network sockets and connections, open files handles, and user/group contexts. The techniques discussed in this chapter are used solely for malicious purposes, and are used to intercept and modify data as well as frustrate the efforts of investigators through anti-forensics. Specifically, you will see the wide range of capabilities that userland rootkits can implement on an infected system, such as function hooking, global offset table/procedure linkage table (GOT/PLT) overwriting, shellcode and library injection, and process hollowing. You will also be taken through a number of analysis techniques and Volatility plugins that can automatically detect a large variety of these userland rootkit methods.

---

## **Analysis Objectives**

Your objectives are these:

- **Detect shellcode injection**: Attackers can inject shellcode (raw assembly instructions) into processes to modify control flow, and manipulate and intercept data. You will learn how to identify process memory that contains these injected sections.
- **Find shared library injection**: Shared library injection allows loading full ELF binaries into a target process. This method is convenient for attackers from a development perspective (they can write their code in C or C++ instead of assembly), but it leaves more traces in memory than simple shellcode injection.
- **Analyze GOT/PLT overwriting**: As the runtime addresses of symbols are resolved, they are stored within entries of the process GOT. By overwriting these pointers, malware can effectively hook function calls in a very simple and standard manner.
- **Trace inline function hooks**: Inline hooks overwrite code within a legitimate function so that control is transferred to a malicious function staged in memory by the malware. This hooking allows the malware to add, modify, and delete any data processed or returned by the hooked function.

---

---

**NOTE**

If you are not familiar with the ELF file format, we strongly recommend reading the relevant sections of Chapter 20 before continuing. They provide prerequisite knowledge for the attack techniques and Volatility plugins described in this chapter.

---

## Shellcode Injection

A block of shellcode is binary data that contains CPU instructions that serve a specific purpose such as spawning a shell (for example, `/bin/bash`), creating a network socket for backdoor access, and/or downloading additional shellcode or executables. Attackers often inject shellcode into legitimate processes as a means of stealth. To write shellcode into a target process and have it executed, the following steps must be performed. Over the next few pages, we’ll cover each step in detail.

1. An open handle must be obtained to the target process.
2. A memory region within the target process that is both writable and executable must be found or allocated.
3. The shellcode must then be written to the previously discovered region.
4. The target process or thread must be forced to execute the given shellcode.

### Step 1: Attaching to a Process with PTrace

To obtain a handle to the target process, attackers leverage Linux’s debugging API (`ptrace`). `ptrace` allows a process with sufficient privileges to have full control over another process, including read/write access to its memory and the ability to set thread execution contexts. The API is exposed through the `ptrace` function whose first argument is the action to perform, second argument is the target process ID, and third argument is the address in the process to act upon. The final argument depends on the action being performed. Here are the relevant actions for performing code injection:

- `PTRACE_ATTACH`: Attaches to the target process and pauses it. This action returns a handle to the target process if successful, which completes the first step in the injection procedure.
- `PTRACE_PEEKTEXT`: Reads data from the address space of the controlled process. It is similar to the Microsoft `ReadProcessMemory` API.
- `PTRACE_POKETEXT`: Writes data into the address space of the target process. It is similar to the Microsoft `WriteProcessMemory` API.
- `PTRACE_GETREGS`: Reads the general-purpose registers from the controlled process.
- `PTRACE_SETREGS`: Sets any of the general-purpose registers of the controlled process.
- `PTRACE_CONT`: Continues a paused process.
- `PTRACE_STOP`: Stops (or pauses) a running process so that it can be examined.
- `PTRACE_DETACH`: Detaches the controlling process from the target process.

### Step 2: Finding/Allocating Memory

After malware has attached to a target process, the shellcode then needs to be transferred to the target’s address space. Malware accomplishes this by overwriting existing code in the target process or by injecting a small block of shellcode into the target process that allocates a larger memory region.

#### Overwriting Existing Code

In many cases, it is possible to find a *hole* within an executable region of memory. These holes, or unused/slack areas, exist because sections of a binary are aligned on a page boundary (typically 4,096 bytes), and the legitimate code of the application may occupy only a small portion of that size. This slack space can leave room for small- to medium-sized payloads, which is perfect because shellcode is typically quite small. Furthermore, by hijacking an already executable region of memory, attackers don’t need to worry about non-executable pages (for example, NX protection).

Another method for finding where to inject the shellcode in a target process was published by skape (see the *Uninformed Research* journal here: [http://hick.org/code/skape/papers/needle.txt](http://hick.org/code/skape/papers/needle.txt)). Instead of finding holes within executable regions, his technique overwrote the beginning of a function in the target process that executes only once, such as `main`. Thus, attackers can avoid overwriting instructions that might actually be referenced again.

#### Foreign Memory Allocation

To use payloads more than a handful of bytes in size, malware must be able to allocate large regions within the target process. Because Linux does not provide a function for allocating memory in a foreign process, such as Microsoft’s `VirtualAllocEx`, the malicious program must typically allocate memory in two steps. In particular, it uses one of the aforementioned techniques to inject a small shellcode stub whose sole purpose is to allocate a larger memory region by calling `mmap` after it is inside the target process’ address space.

Historically, malware used `malloc` to allocate memory for shellcode on the heap, but 32-bit Physical Address Extension (PAE) and 64-bit systems contain non-executable stacks and heaps. Thus, on modern systems, the only function available that meets the necessary allocation criteria is `mmap`, which allows for not only allocating memory but also setting its page permissions. By using `mmap`, the malware can allocate memory and specify the protection as readable, writable, and executable. Once the first stage shellcode calls `mmap` within the target process, the real payload can then be written to the newly allocated (large) region and executed.

---

**NOTE**

The inability to allocate memory as writable and executable is one of the restrictions imposed by the `MPROTECT` feature of the PaX-kernel-hardening patches from Grsecurity ([http://pax.grsecurity.net/docs/mprotect.txt](http://pax.grsecurity.net/docs/mprotect.txt)). From the documentation guide, the operations prevented are these:

- Creating executable anonymous mappings
- Creating executable/writable file mappings
- Making an executable/read-only file mapping writable except for performing relocations on an `ET_DYN ELF` file (non–position independent code [PIC] shared library)
- Making a non-executable mapping executable

These restrictions prevent processes from allocating regions that could potentially contain executable shellcode.

---

### Step 3: Writing Shellcode into a Process

After the malware finds a suitable location (using one of the previous methods), `PTRACE_POKETEXT` can be used to write the shellcode into the foreign process. Since the 3.2 kernel version, Linux has provided two functions, `process_vm_readv` and `process_vm_writev`, that allow for reading and writing of a foreign process’ memory without the use of `ptrace`. Although it means that the use of `PTRACE_PEEKTEXT` and `PTRACE_POKETEXT` could be avoided, all the other `ptrace` functionality in this section is still required. Furthermore, because these system calls are relatively new at the time of writing, we have yet to see any malware in the wild use them.

### Step 4: Controlling Foreign Process Execution

After the shellcode exists in the target process’ memory, the malware must force it to execute. The easiest way to redirect program execution to the shellcode is to use the following algorithm:

1. Call `ptrace` with the `PTRACE_GETREGS` operation, which pauses the program and gives you a copy of all registers and their current values.
2. Overwrite the instruction pointer register so that it points to the shellcode.
3. Use `PTRACE_SETREGS` to update the registers within the foreign process.
4. Use `PTRACE_CONT` to resume the process.

When the program resumes, it then executes the injected shellcode. Of course, this method of overwriting can be destructive if executed within a critical thread and the instruction pointer is not reset after the shellcode block executes.

Another method to force execution is to read the current value of the instruction pointer, back up the instructions currently after it, copy the shellcode to the address pointed to by the instruction pointer, and then let the shellcode execute. If the shellcode is made to execute an `int 3` (software breakpoint) instruction after completing its tasks, the malware process takes control again. The malware process can then restore the backed-up instructions and let them execute. In this manner, the malicious code acts just like a debugger.

### Detecting Shellcode Injection

You can use the `linux_malfind` plugin to find shellcode that was injected into processes by allocating new regions. You can use the `linux_hollow_process` plugin, described later in this chapter, to detect skape’s technique of overwriting part of the existing `main` function with shellcode.

`linux_malfind` works similarly to its Windows counterpart (as described in the “Code Injection” section of Chapter 8) because it walks the memory mappings of a process looking for suspicious protection bits. In particular, it looks for mappings whose protection bits indicate that the region is readable, writable, and executable. This condition should not occur using standard process-loading mechanisms, but it might occur in userland applications that need to generate code at runtime, such as interpreters of dynamic languages (Perl, Python, and JavaScript).

To demonstrate how to use `linux_malfind`, a program named `inject` was used to inject code into a process named `target`. The injected payload spawns a backdoor shell that listens on TCP port 31337. The `inject` application uses the two-step procedure previously described. First, a small block of shellcode that calls `mmap` is injected by overwriting the beginning of the target process’ `main` function. This `mmap` shellcode allocates a page of memory at a hardcoded address (`0x1011000`) that is readable, writable, and executable. It then writes the payload into the allocated memory and executes it.

`linux_malfind` immediately picks up this injected segment due to its invalid protection bits:

```
$ python vol.py -f injtarget.lime --profile=LinuxDebian3_2x86 linux_malfind
Volatility Foundation Volatility Framework 2.4
Process: target Pid: 16929 Address: 0x1011000 File: Anonymous Mapping
Protection: VM_READ|VM_WRITE|VM_EXEC
Flags: VM_READ|VM_WRITE|VM_EXEC|VM_MAYREAD|VM_MAYWRITE|VM_MAYEXEC|VM_ACCOUNT

0x01011000  31 c0 31 db 31 c9 31 d2 b0 66 b3 01 51 6a 06 6a   1.1.1.1..f..Qj.j
0x01011010  01 6a 02 89 e1 cd 80 89 c6 b0 66 b3 02 52 66 68   .j........f..Rfh
0x01011020  7a 69 66 53 89 e1 6a 10 51 56 89 e1 cd 80 b0 66   zifS..j.QV.....f
0x01011030  b3 04 6a 01 56 89 e1 cd 80 b0 66 b3 05 52 52 56   ..j.V.....f..RRV

0x1011000 31c0             XOR EAX, EAX
0x1011002 31db             XOR EBX, EBX
0x1011004 31c9             XOR ECX, ECX
0x1011006 31d2             XOR EDX, EDX
0x1011008 b066             MOV AL, 0x66
0x101100a b301             MOV BL, 0x1
0x101100c 51               PUSH ECX
0x101100d 6a06             PUSH 0x6
0x101100f 6a01             PUSH 0x1
0x1011011 6a02             PUSH 0x2
0x1011013 89e1             MOV ECX, ESP
0x1011015 cd80             INT 0x80
0x1011017 89c6             MOV ESI, EAX
0x1011019 b066             MOV AL, 0x66
0x101101b b302             MOV BL, 0x2
0x101101d 52               PUSH EDX
0x101101e 66687a69         PUSH WORD 0x697a
<snip>
```

As shown in the output, the target process (PID 16929) has a region of memory at address `0x1011000` that is readable, writable, and executable. The `mmap` shellcode initially allocates this region. You can also use `linux_netstat` to see that the shellcode successfully created the backdoor network socket:

```
$ python vol.py -f injtarget.lime --profile=LinuxDebian3_2x86 
   linux_netstat -p 16929
Volatility Foundation Volatility Framework 2.4
TCP      0.0.0.0:31337 0.0.0.0:0     LISTEN            target/16929
```

In an investigation, you could now begin to study the disassembly of the shellcode to determine which capabilities it provides for the attacker.

## Process Hollowing

Process hollowing is the act of overwriting a process or portions of a process in memory with malicious code. Attackers often use process hollowing to hide the presence of malware on a system because tools that list processes see a legitimate process instead. Also, the in-kernel data structures for the process will report the path to the original process’ binary (for example, `/usr/bin/apache2`), not to the malware’s file. This misdirection makes antivirus and other host-protection tools scan innocent files on disk and lets the malware further evade detection.

---

**NOTE**

On the Windows side, infamous malware samples, such as Stuxnet and Careto (The Mask), used process hollowing to evade detection on live systems.

---

### The Detection Algorithm

Process-hollowing detection relies on having a known good copy of an application’s instructions to compare with those in memory. Although you *could* consult the kernel’s file cache (see Chapter 24) to obtain a copy of the file as it appears on disk, the desired file might not exist in the cache or it might be incomplete (due to swapping). Furthermore, you cannot trust that files on a compromised system’s disk are the original, unmodified versions.

Because of these issues, Volatility relies on a copy of the file from a baseline disk image (or installation package) to obtain the known good data. Thus, provided that you have the ability to obtain a trusted copy of the potentially “infected” file(s), you can use the `linux_process_hollow` plugin to perform the check. This plugin requires three options:

- The path to the trusted binary file
- The PID(s) to operate on
- The address that the application is mapped into the process’ memory

The plugin starts by reading the symbol table of the trusted file and determining where each function is loaded into memory. It then compares each function with the actual code in memory. Except for the case of direct relocations, which do not use the GOT/PLT (they overwrite function addresses at runtime), the code of an application should not change. Volatility detects the direct relocations by parsing the relocation table and accounts for them to avoid false positives.

---

**NOTE**

The current implementation only operates on one process at a time. However, remember that Volatility is open source and extensible. You can update the code so that it obtains the full paths on disk to loaded processes, shared libraries, and kernel modules; then automate the comparison with the corresponding files from your baseline disk image or install media.

---

### An Example of Detection

The following scenario involves a system infected with malware that performs process hollowing (and also shared library injection, which we discuss in the next section). The target process is an instance of bash and its PID is 18550, as the first command shows:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_pslist | grep bash
Volatility Foundation Volatility Framework 2.4
Offset     Name    Pid   Uid   Gid  DTB        Start Time
------------------ ----- ----- ---- ---------  ----------------------------
0xf7116140 bash   18550 0     0    0x35acd000 2014-02-25 12:09:19 UTC+0000
```

The next command determines the address of where the target ELF file is loaded in process memory. According to the output, the executable region (`r-x`) starts at `0x80480000`. You also see the full path to the binary on disk (`/bin/bash`).

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_proc_maps -p 18550 | grep bash
Volatility Foundation Volatility Framework 2.4
Pid   Start     End       Flags Pgoff Major Minor  Inode  File Path
----- --------- --------- ----- ----  ----- ------ ------ -------------------
18550 0x8048000 0x8049000 r-x   0x0   8     1      948592 /bin/bash
18550 0x8049000 0x804a000 rw-   0x0   8     1      948592 /bin/bash
```

We then pass this information to `linux_process_hollow` along with the path to the trusted copy of the executable (`/mnt/baselines/bin/bash`). The plugin reports that the `main` function has been altered:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_process_hollow -p 18550 
       -b 0x8048000 –P /mnt/baselines/bin/bash
Volatility Foundation Volatility Framework 2.4
Task     PID  Symbol Name Symbol Address
-------- ---- ----------- --------------
bash    18550 main        0x80485bc
```

If we investigate the symbol address with `linux_volshell`, we find a block of instructions and a string reference to `/tmp/.XICE-unix`:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 linux_volshell
Volatility Foundation Volatility Framework 2.4
Current context: process init, pid=1 DTB=0x370bf000
Welcome to volshell! 
To get help, type 'hh()'
>>> cc(pid=18550)
Current context: process bash, pid=18550 DTB=0x35acd000
>>> db(0x80485bc, 48)
0x080485bc  eb 17 58 89 04 24 c7 44 24 04 01 00 00 00 bb e0   ..X..$.D$.......
0x080485cc  3a 75 b7 ff d3 83 c4 04 cc e8 e4 ff ff ff 2f 74   :u............/t
0x080485dc  6d 70 2f 2e 58 49 43 45 2d 75 6e 69 78 00 00 00   mp/.XICE-unix...
>>> dis(0x80485bc)
0x80485bc eb17                             JMP 0x80485d5
0x80485be 58                               POP EAX
0x80485bf 890424                           MOV [ESP], EAX
0x80485c2 c744240401000000                 MOV DWORD [ESP+0x4], 0x1
0x80485ca bbe03a75b7                       MOV EBX, 0xb7753ae0
<snip>
```

A `JMP` instruction has replaced code that should correspond to the `main` function. Typically you would see a function prologue such as `PUSH EBP`, followed by `MOV EBP, ESP`. In this case, however, the CPU is redirected to the code at address `0x80485d5`, which is further along in the injected shellcode. The exact operation of this shellcode is explained in the following section.

## Shared Library Injection

Injecting shellcode into a process is very useful for attackers during initial exploitation and for *limited* post-compromise activities. However, writing full-featured software purely in shellcode is difficult because it must be position-independent (that is, no hardcoded addresses) and cannot directly rely on API calls. To remedy this situation, many programmers choose to implement their backdoors as shared libraries instead, which they can write in C.

As this section describes, attackers have two methods in which to inject libraries into a foreign process. The first method, which is the simplest, uses the native system APIs to load the attacker’s shared library stored on disk into the address space of another process. However, this technique leaves a number of traces throughout memory and disk. The second method loads a library that is located only in memory and never writes it to disk. This method is more difficult to implement, but it leaves far fewer artifacts in memory and no artifacts on disk.

### Injecting a Library from Disk

The Linux dynamic loader provides functions such as `dlopen`, `dlclose`, and `dlsym`, which are equivalent to the Windows functions `LoadLibrary`, `FreeLibrary`, and `GetProcAddress`, respectively. Together they allow programmers to load libraries, unload libraries, and query symbol addresses within libraries at runtime. To inject a shared library stored on disk into a foreign process, all the malicious process must do is force the target process to call `_dlopen` (a function that `dlopen` wraps) with its first parameter set to the path of the shared library.

---

**NOTE**

`_dlopen` is used instead of `dlopen` because `dlopen` is present only in applications that programmatically interact with the runtime loader. On the other hand, `_dlopen` is within `libc` and loads into the address space of all dynamically linked applications. On some recent versions of `libc`, `_dlopen` has also been deprecated and replaced with `__libc_dlopen_mode`. The same process can be used to call this function in all dynamically linked applications and is used by projects such as injectso64 ([https://github.com/ice799/injectso64](https://github.com/ice799/injectso64)).

---

To the best of our knowledge, the first public documentation of this technique was in *Runtime Process Infection* ([http://phrack.org/issues/59/8.html](http://phrack.org/issues/59/8.html)). In this article, the author used the previously described `ptrace` technique to inject shellcode into the process and then execute it. The shellcode subsequently loaded a library like this:

```
1   _start:    jmp string
2   
3   begin:     pop    eax                   ; char *file
4              xor    ecx     ,ecx          ; *caller
5              mov    edx     ,0x1          ; int mode
6
7              mov    ebx,    0x12345678    ; addr of _dl_open()
8              call   ebx                   ; call _dl_open!
9              add    esp,    0x4
10  
11             int3                         ; breakpoint
12  
13  string: call begin
14          db "/tmp/ourlibby.so",0x00
```

On line 1, the shellcode jumps to the `string` label on line 13. The `call begin` instruction then transfers control back up to line 3. This redirection has the effect of placing the address of the `/tmp/ourlibby.so` string (the path of the library to be loaded) on the top of the stack. Thus, when line 3 performs a `pop eax`, the string’s address is copied into the `EAX` register and becomes the first argument to `_dl_open`. Line 7 holds a dummy value for the address of `_dl_open`, which is patched at runtime with the real address of the function within the target process. On line 8, `_dl_open` is called, and on line 11 `int 3` is executed to return control to the malicious process.

#### Detecting Disk–Based Shared Library Injections

You can detect disk–based shared library injections by using a number of methods. Because the malicious library is loaded in the same manner (that is, with `_dlopen`) as legitimate libraries, the kernel populates data structures for the mapped file. Thus, you can use the `linux_proc_maps` plugin for detection. In this case, malware injected a library named `/tmp/.XICE-unix` into the target process. This filename is intentionally misleading because most Linux distributions have a directory named `/tmp/.ICE-unix` (created by the X server):

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_proc_maps -p 18550
Volatility Foundation Volatility Framework 2.4
Pid      Start    End         Flags  Pgoff Major  Minor  Inode     File Path
-------- -------- ----------- ------ ----- ----- ------ ---------- -------
<snip>
   18550 0xb77b1000 0xb77b2000 r-x   0x0   8      1     1439419 /tmp/.XICE-unix
   18550 0xb77b2000 0xb77b3000 rw-   0x0   8      1     1439419 /tmp/.XICE-unix
<snip>
```

---

**NOTE**

Linux makes two mappings for each mapped application or library. One is for the executable data (note the `r-x` protection), and the other is for the data mapping (`rw-` protection).

---

In this output, an abnormal library has obviously been mapped into the process because it loads from `/tmp` instead of `/lib` or `/usr/lib`. Although different applications might load libraries from outside of the `lib` directories, you typically won’t see them in temporary paths. Another way to spot discrepancies is to filter these entries based on a whitelist of known good values. For example, create baselines of disk images and modify the plugin to alert only on libraries not in your baseline.

#### Listing Libraries in Userland

Volatility has a plugin, `linux_library_list`, which is specifically designed to report libraries mapped into a process. In particular, it analyzes the list of loaded libraries that the dynamic linker in userland maintains. This is very similar to the list of dynamic link libraries (DLLs) kept in the Process Environment Block (PEB) on Windows (see Chapters 7 and 8). In this list, each mapping is represented by a `link_map` structure, which maintains the starting address and file system path of libraries mapped using functions such as `dlopen`. The presence and enumeration of this list was first presented by the grugq ([http://www.ouah.org/melfbuggery.html](http://www.ouah.org/melfbuggery.html)).

The following command shows `linux_library_list` against the process infected by the `/tmp/.XICE-unix` malicious library:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_library_list -p 18550
Volatility Foundation Volatility Framework 2.4
Task             Pid      Load Address Path
---------------- -------- ------------ ----
bash              18550   0x00b7647000 /lib/i386-linux-gnu/i686/cmov/libc.so.6
bash              18550   0x00b77b7000 /lib/ld-linux.so.2
bash              18550   0x00b77b1000 /tmp/.XICE-unix
[snip]
```

In this output, you can see the shared libraries loaded into the process, including the malicious library.

#### Cross-Referencing Mappings

The `linux_ldrmodules` plugin automatically cross-references libraries found through the kernel’s list of per–process memory mappings and the doubly linked list of libraries kept by the dynamic linker. This plugin can detect malware that hides by manipulating the dynamic linker’s list (a common technique used to hide from debuggers and other live tools). Again, this attack is very similar to malware tactics seen on Windows systems that alter the list of DLLs kept in userland. You can also use the `linux_ldrmodules` plugin to detect libraries injected solely in memory (discussed in the next section).

In order to determine which of the kernel mappings hold shared libraries, the plugin checks the beginning of each mapping for the ELF file header. If the ELF header is found and the mapping protections have the execute bit set, the mapping is reported. This has the side effect of finding all ELF files, including the main process. The following shows `linux_ldrmodules` against the process with the `.XICE-unix` library injected:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_ldrmodules -p 18550
Volatility Foundation Volatility Framework 2.4
Pid   Name    Start      File Path                       Kernel  Libc
----- ------- ---------  -------------------             ------  ------
18550 bash    0x08048000 /bin/bash                        True   False
18550 bash    0xb7647000 /lib/[snip]/cmov/libc-2.13.so    True   True
18550 bash    0xb77b7000 /lib/i386-linux-gnu/ld-2.13.so   True   True
18550 bash    0xb77b1000 /tmp/.XICE-unix                  True   True
[snip]
```

The binary for the process (`/bin/bash`) is found only in process mappings. This is normal because it is not loaded through the dynamic linker. However, if you see `False` in the `Libc` column for any entry other than the process’ main binary, you should flag it for further investigation. You can always determine which entry corresponds to the process by matching the `task.mm.start_code` address with the address of the mapping.

As expected, the remaining entries are found in both lists, which indicates exactly how they were loaded (via `dlopen`). It also tells you that they aren’t hidden from the system in additional ways. Right now, `.XICE-unix` is hiding in plain sight, so to speak. If it tried to be extra stealthy by unlinking from the dynamic linker list in userland, you’d then see a discrepancy in this plugin’s output.

### Injecting a Library from Memory

To avoid the artifacts left by disk-based injections, particularly the library file and the corresponding `/proc/<pid>/maps` entries, attackers designed methods that can load applications and libraries directly from memory. When this technique was developed, no memory analysis capabilities existed within the forensics community. Thus, if attackers could avoid leaving artifacts on disk, they could often completely bypass detection by normal system administration tools as well as forensic analysis. The first papers to publicly explore this idea on Linux were the following:

- *The Design and Implementation of Userland Exec* by the grugq: [https://github.com/grugq/grugq.github.com/blob/master/docs/ul_exec.txt](https://github.com/grugq/grugq.github.com/blob/master/docs/ul_exec.txt). This technique focused on imitating the functionality of `execve` entirely in user mode, which allowed processes to be created without involving the kernel. Thus, an application could be launched right off the network instead of being written to disk first. Despite being published in 2004, this technique to execute an application solely in memory is still used in the wild.
- *Remote Library Injection* by skape: [http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf](http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf). This technique involves using `mmap` to allocate a readable, writable, and executable memory region in the target process to store the malicious library’s code. It relies on hooking the dynamic loader’s API functions (for example, `open`, `read`, `lseek`, `mmap`, `fxstat64`) so that they work on the binary already loaded into memory instead of a file on disk. It then calls `_dl_open`, which runs using modified routines.

You can detect memory-only injections in a number of ways. The first is by using `linux_malfind`. As discussed in “Detecting Shellcode Injection,” the mappings that contain shellcode must be writable and executable. Thus, `linux_malfind` will flag this suspicious set of protections. All the methods published for loading an executable directly from memory use shellcode to at least allocate a suitable memory region in the target process, if not for other purposes as well.

Even if the malware frees its shellcode region after its library is loaded, you can detect the library in several ways. For example, skape’s technique uses `_dl_open` (albeit a hooked version), so it still populates the list of libraries that the loader keeps, making the library visible to `linux_library_list`. If the loader’s list of libraries is tampered with, `linux_ldrmodules` flags the malware because the injected library appears as `True` in kernel mappings, but `False` in the dynamic linker’s mappings. A similar discrepancy exists if the `execve` method described by the grugq is used because it does not rely on calling `_dl_open`. Thus, the injected application is never created in the dynamic linker list in the first place. However, it contains an ELF header, so the `linux_ldrmodules` method can find it.

### Extracting Executables from Memory

You will often want to extract executables and shared libraries from memory to statically analyze them. To automate this process, the `linux_procdump` and `linux_librarydump` plugins were created to mimic their Windows counterparts, `procdump` and `dlldump`.

The main purpose of these plugins is to extract executables in their native ELF form, not just as regions of disjointed memory, which would happen if you use `linux_dump_maps`. To accomplish this, the Volatility API provides a function that takes a `task_struct` and the virtual address of an ELF header as parameters. It then acquires the process address space of the given `task_struct` and instantiates an `elf_hdr` object at the given virtual address.

To find the sections to extract, the program headers of the file are enumerated, and those of type `PT_LOAD` are saved. `PT_LOAD` sections are loaded into memory upon program execution or library initialization. If you examine the program headers of a file on disk, you see that all the important sections are contained within one of two `PT_LOAD` sections. In a normal ELF binary, one section is for executable data and the other is for writable data:

```
$ readelf -Wl /bin/ls

Elf file type is EXEC (Executable file)
Entry point 0x404880
There are 9 program headers, starting at offset 64

Program Headers:
  Type           Offset   VirtAddr PhysAddr FileSiz  MemSiz   Flg Align
  PHDR           0x000040 0x400040 0x400040 0x0001f8 0x0001f8 R E 0x8
  INTERP         0x000238 0x400238 0x400238 0x00001c 0x00001c R   0x1
       [Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]
LOAD           0x000000 0x400000 0x400000 0x01a60c 0x01a60c R E 0x200000
LOAD           0x01adb0 0x61adb0 0x61adb0 0x0007cc 0x001550 RW  0x200000
  DYNAMIC        0x01adc8 0x61adc8 0x61adc8 0x000210 0x000210 RW  0x8
  NOTE                 0x000254 0x400254 0x400254 0x000044 0x000044 R   0x4
  GNU_EH_FRAME   0x017e28 0x417e28 0x417e28 0x00070c 0x00070c R   0x4
  GNU_STACK      0x000000 0x000000 0x000000 0x000000 0x000000 RW  0x8
  GNU_RELRO      0x01adb0 0x61adb0 0x61adb0 0x000250 0x000250 R   0x1

  Section to Segment mapping:
   Segment Sections...
    00
    01     .interp
02     .interp .note.ABI-tag .note.gnu.build-id .hash .gnu.hash .dynsym 
    .dynstr.gnu.version .gnu.version_r .rela.dyn .rela.plt .init  
    .plt .text .fini .rodata .eh_frame_hdr .eh_frame
    03     .init_array .fini_array .jcr .dynamic .got .got.plt .data .bss
    04     .dynamic
    05     .note.ABI-tag .note.gnu.build-id
    06     .eh_frame_hdr
    07
    08     .init_array .fini_array .jcr .dynamic .got
```

In the Program Headers portion, you see two `LOAD` segments in bold. These two entries correspond to segments 02 and 03 in the Section to Segment mapping section, since the mappings leverage zero-based numbering. The first `LOAD` segment is readable and executable, and its section-to-segment mapping includes expected sections such as `.text`, `.plt`, and `.init`. The other `LOAD` segment is the readable and writable one, and its section-to-segment mapping includes the `.data` and `.bss` sections. So when executables are extracted from memory, the `LOAD` segments will have nearly all of the important sections. The ones missing are those useful for debugging but not needed at runtime, such as the static symbol table and section information.

The following commands show how to use `linux_librarydump` to recover the `.XICE-unix` shared library:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_proc_maps -p 18550 | grep ICE
Volatility Foundation Volatility Framework 2.4
   18550 0xb77b1000 0xb77b2000 r-x   0x0   8   1    1439419 /tmp/.XICE-unix
   18550 0xb77b2000 0xb77b3000 rw-   0x0   8   1    1439419 /tmp/.XICE-unix

$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_librarydump -p 18550 
       -D outdir -b 0xb77b1000
Volatility Foundation Volatility Framework 2.4
Offset     Name        Pid       Address    Output File
---------- ----------- --------- ---------- -----------
0xf7116140 bash        18550     0xb77b1000 outdir/bash.18550.0xb77b1000

$ file outdir/bash.18550.0xb77b1000
outdir/bash.18550.0xb77b1000: ELF 32-bit LSB shared object, Intel 80386, 
version 1 (SYSV), dynamically linked, 
BuildID[sha1]=0x8200585e3443980847707c3f63eaa38f531fb67a, not stripped
```

The library is extracted from memory successfully and can then be analyzed using reverse-engineering techniques.

## LD_PRELOAD Rootkits

You can achieve a special version of shared library injection by using the `LD_PRELOAD` facility provided by the dynamic loader. `LD_PRELOAD` is an environment variable that specifies the path to a shared library. Any program run with `LD_PRELOAD` set (except for those that are `setuid` for a different or higher privilege than the user) has the specified library loaded into its address space first. It is then easy to hook API functions from any other library within the process’ address space because symbols are resolved within the preloaded library first. Itzik Kotler first published this technique in his article, *Reverse Engineering with LD_PRELOAD*: [http://securityvulns.com/articles/reveng](http://securityvulns.com/articles/reveng).

### Hooking Functions with LD_PRELOAD

The following code shows an example of a simple library that could be used in conjunction with the `LD_PRELOAD` technique to hook the `write` library call. This hook allows the rootkit to intercept data as it is written to disk, across the network, or to an output terminal:

```
  1 #include <stdio.h>
  2 #include <dlfcn.h>
  3 #include <unistd.h>
  4 #include <fcntl.h>
  5
  6 ssize_t (*orig_write)(int, const void *, size_t);
  7 int log_fd;
  8
  9 ssize_t write(int fd, const void *buf, size_t count)
 10 {
 11     void *handle;   
 12
 13     if (!orig_write)
 14     {
 15         handle = dlopen("libc.so.6", RTLD_LAZY);
 16         orig_write = dlsym(handle, "write");
 17         log_fd = open("/tmp/logfile.txt", O_CREAT|O_WRONLY, 0666);
 18     }
 19
 20     orig_write(log_fd, buf, count);
 21     return orig_write(fd, buf, count);
 22 }
```

Line 9 declares the fake `write` function. On line 15, it obtains a handle to `libc` in memory, which is the library that contains the legitimate `write` function. The next line then resolves the runtime address of the real `write` function by calling `dlsym`. This address is needed so that the real function can still be called when necessary. Line 17 opens a handle to the log file (`/tmp/logfile.txt`). On line 20, the buffer passed to `write` is logged to the malware’s log file, and on line 21 the original write is called with the given parameters. Applications using `write` are not notified of this modification, so they are unaware that the malware is recording all data. The same process can be applied to any function, such as the `read` call, in order to log data from the network or user input.

### Detecting LD_PRELOAD Rootkits

Rootkits that use `LD_PRELOAD` can be detected in a number of ways. To demonstrate, we’ll use a memory sample provided by the Second Look project ([http://secondlookforensics.com/linux-memory-images/](http://secondlookforensics.com/linux-memory-images/)). In particular, the machine was infected with Jynx2 ([http://www.blackhatlibrary.net/Jynx_Rootkit/2.0](http://www.blackhatlibrary.net/Jynx_Rootkit/2.0)), which is a popular open-source `LD_PRELOAD`-based rootkit.

---

**NOTE**

Azazel ([http://blackhatlibrary.net/Azazel](http://blackhatlibrary.net/Azazel)) is another open-source rootkit that uses the `LD_PRELOAD` trick.

---

#### Extracting Preload Files

Rootkits can force every process to preload a library by writing the path of the library to `/etc/ld.so.preload`. The dynamic loader checks this file when starting applications, and it subsequently loads the requested library; Jynx2 and Azazel both take this approach. To detect it, you can simply use `linux_find_file` or `linux_recover_filesystem` to read `/etc/ld.so.preload` from memory and then examine its contents:

```
$ python vol.py --profile=LinuxUbuntu1204x64 -f jynxkit.mem 
       linux_find_file -F /etc/ld.so.preload
Volatility Foundation Volatility Framework 2.4
Inode Number                  Inode
---------------- ------------------
          263883 0xffff88003be9b440

$ python vol.py --profile=LinuxUbuntu1204x64 -f jynxkit.mem 
       linux_find_file -i 0xffff88003be9b440 
       -O ld.so.preload
Volatility Foundation Volatility Framework 2.4

$ cat ld.so.preload
/XxJynx/jynx2.so
```

The full path to the malicious Jynx library (`/XxJynx/jynx2.so`) is found within the preload file. On normal systems, libraries shouldn’t typically be preloaded via this method. Thus, checking the content of `ld.so.preload` files can be a simple indicator of compromise. The following command searches all process mappings for the malicious library and counts the number of occurrences:

```
$ python vol.py --profile=LinuxUbuntu1204newx64 -f jynxkit.mem
     linux_proc_maps | grep -c /XxJynx/jynx2.so
Volatility Foundation Volatility Framework 2.4
364
```

This command shows that the Jynx library is mapped 364 times by various processes on the system.

#### Analyzing Preload Variables

A second method to force a shared library to load is by setting the `LD_PRELOAD` environment variable within a user’s bash preferences file (typically `.bashrc` or `.bash_profile`). As discussed in Chapter 21, this same technique was used to change the `PATH` variable of a user to force the execution of trojan binaries. Just as the `PATH` tampering can be detected with `linux_bash_env`, so can the `LD_PRELOAD` trick. The following example shows how this attack appears by examining an infected `netcat` process:

```
$ python vol.py -f ncpreload.lime --profile=LinuxDebian3_2x86 
    linux_psenv -p 14259
Volatility Foundation Volatility Framework 2.4
Name   Pid    Environment
nc     14259  TERM=xterm SHELL=/bin/bash SSH_CLIENT=192.168.174.1 51514 22 
LD_PRELOAD=/root/ldpre/netlib.so OLDPWD=/root SSH_TTY=/dev/pts/2 USER=root 
MAIL=/var/mail/root PATH=/root/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:
/usr/bin:/sbin:/bin PWD=/root/ldpre LANG=en_US.UTF-8 SHLVL=1 HOME=/root 
LOGNAME=root SSH_CONNECTION=192.168.174.1 51514 192.168.174.128 22 
_=/bin/nc /bin/nc 
```

`LD_PRELOAD` points to the malicious file on disk. You can easily automate finding all instances of this variable and subsequently invoke the `linux_librarydump` plugin to extract the libraries.

#### Checking the GOT/PLT

The next method to detect `LD_PRELOAD`-based rootkits involves checking for GOT/PLT overwrites. This process will be explained more in the next section, but the idea is that the GOT holds the addresses of functions that an application uses and can be tampered with to redirect control flow. This type of hooking is very similar to import and export address table hooks in Windows. Because the preloaded library is loaded first, the function pointers within the table point to the preloaded library, not the original (for example, `libc`).

#### Comparative Symbol Analysis

A detection method that can find `LD_PRELOAD` rootkits on a live system was posted on the Chokepoint blog in February 2014 ([http://www.chokepoint.net/2014/02/detecting-userland-preload-rootkits.html](http://www.chokepoint.net/2014/02/detecting-userland-preload-rootkits.html)). This detection method is interesting because it relies on the dynamic linker to determine inconsistencies. It works in the following manner:

1. It uses the `dlsym` function to request the address of a number of commonly hooked functions.
2. It then requests the same function from `dlsym` with the `RTLD_NEXT` flag set. This request tells the dynamic loader to skip the first library in which the symbol is found and look for it in the rest of the loaded libraries.
3. It compares the two results and determines whether a mismatch exists.

Keep in mind that each function should be exported by only one library. Thus, an obvious inconsistency occurs if multiple libraries export the same symbol. Using the Volatility library listing and symbol resolution API, this same technique could easily be ported into a plugin.

## GOT/PLT Overwrites

As discussed in Chapter 20, the GOT and PLT are involved in the mechanism that allows an application to call functions stored within other libraries. After symbol resolution, the entries within the GOT store the full runtime address of the resolved symbol. By overwriting these entries, malware can redirect calls to legitimate functions to code that malware controls. This redirection allows malware to manipulate any data that’s processed by the redirected functions.

### PLT Tampering

The first research publication that showed how to perform PLT overwriting was *Shared Library Redirection Via ELF PLT Infection* by Silvio Cesare ([http://phrack.org/issues/56/7.html](http://phrack.org/issues/56/7.html)). This article focused on infecting entries within a file on disk; however, the technique has also been used against ELF files in memory.

---

**NOTE**

Many exploits gain the capability to write to arbitrary addresses in memory and then target GOT entries to redirect control flow. For more information, see the following research papers:

- *Advances in Format String Exploitation* by gera and riq: [http://phrack.org/issues/59/7.html](http://phrack.org/issues/59/7.html)
- *w00w00 on Heap Overflows* by Matt Conover: [http://www.cgsecurity.org/exploit/heaptut.txt](http://www.cgsecurity.org/exploit/heaptut.txt)
- *Hackers Hut: Exploiting the Heap* by Andries Brouwer: [http://www.win.tue.nl/~aeb/linux/hh/hh-11.html](http://www.win.tue.nl/~aeb/linux/hh/hh-11.html)

---

Overwriting a GOT entry requires three capabilities:

- To read and write data in a foreign process
- To find the target GOT entry
- To find the address of the hook function within the target process’ address space

You have already learned how to read and write a process’ memory with `ptrace`. You also know that you can use `dlopen` and `dlsym` to find the address of any exported function within a module. The only thing that has not been covered yet is locating the GOT entry of the target function. To find this information, you can query the relocation table. Each relocation entry lists the address of a resolved function within the GOT. For more information, use the `readelf` command with the `-r` parameter (which causes relocation information to be displayed). Here’s an example:

```
1   $ readelf -W -r test_app
2
3   Relocation section '.rel.dyn' at offset 0x324 contains 1 entries:
4   Offset     Info    Type                Sym. Value  Symbol's Name
5   08049838  00000506 R_386_GLOB_DAT         00000000   __gmon_start__
6
7   Relocation section '.rel.plt' at offset 0x32c contains 8 entries:
8   Offset     Info    Type                Sym. Value  Symbol's Name
9   08049848  00000107 R_386_JUMP_SLOT        00000000   strstr
10  0804984c  00000207 R_386_JUMP_SLOT        00000000   read
11  08049850  00000307 R_386_JUMP_SLOT        00000000   perror
12  08049854  00000407 R_386_JUMP_SLOT        00000000   puts
13  08049858  00000507 R_386_JUMP_SLOT        00000000   __gmon_start__
14  0804985c  00000607 R_386_JUMP_SLOT        00000000   exit
15  08049860  00000707 R_386_JUMP_SLOT        00000000   open
16  08049864  00000807 R_386_JUMP_SLOT        00000000   __libc_start_main
```

In this output, the entries of type `R_386_JUMP_SLOT`(lines 9–16) represent imported functions. The `Offset` column tells you the address of the GOT entry for the function. By reading from this address, you can determine the runtime address of each function whose name is given in the `Symbol's Name` column.

### Detecting GOT Overwrites

Detecting GOT overwrites is typically a four-step process:

1. Walk the dynamic linking information of the application and all loaded libraries.
2. Record the resolved address and symbol name for all functions within the relocation tables. For libraries that are lazily loaded, the symbol addresses are not resolved until the function is actually called. Lazy loading can mean that some GOT entries will point inside the containing ELF object, which makes filtering out such entries easy.
3. Identify the libraries that the application *need*s. These libraries are determined at compile time and stored as `DT_NEEDED` entries within the dynamic linking information.
4. Validate each resolved GOT entry by checking that the resolved address points into one of the needed libraries. This validation process ensures that the function points into one of the libraries that the program was linked with at compile time.

The following shows the output of the Volatility GOT/PLT overwrite plugin, `linux_plthook`. In this case, the plugin is used to analyze a Secure Shell (SSH) server process infected by the Jynx2 rootkit:

```
$ python vol.py --profile=LinuxDebian3_2x86 -f preload.lime 
   linux_plthook -p 22996 
Volatility Foundation Volatility Framework 2.4
Task  ELF Start  ELF Name        Symbol         Resolved Address Target Info
22996 0x08048000 /usr/sbin/sshd  __xstat64      0x000000b7743fc9 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  write          0x000000b774327a /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  fopen64        0x000000b7742f32 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  __lxstat64     0x000000b7743930 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  opendir        0x000000b7744432 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  accept         0x000000b7742bf0 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  readdir64      0x000000b7744660 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  unlink         0x000000b77440d9 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  rmdir          0x000000b7743b5c /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  __fxstat64     0x000000b7743532 /root/jynx2.so
```

Several functions within the SSH server’s GOT redirect to addresses within the Jynx shared library. You can use this same approach to generically detect all `LD_PRELOAD` rootkits (which automatically overwrite GOT entries) as well as those that manually overwrite GOT entries.

---

**NOTE**

Georg Wicherski developed the `linux_plthook` plugin to assist with one of his investigations. In this case, he encountered an advanced piece of malware that operated only in memory and used GOT/PLT overwrites to control infected systems. You can find more information on Georg’s analysis of this infection in his SysScan 2014 presentation, *Linux Memory Forensics: A Real-Life Case Study* (see [http://syscan.org/index.php/download/get/7d8da2be5feac35e006f618beda71459e7/SyScan2014_SPEAKER13.zip](http://syscan.org/index.php/download/get/7d8da2be5feac35e006f618beda71459e7/SyScan2014_SPEAKER13.zip)).

---

## Inline Hooking

Instead of replacing function pointers, such as those in the GOT, an inline hook overwrites the first several bytes of a function and replaces them with instructions of the malware’s choosing. These instructions normally redirect control flow to a function the malware places in memory. Malware authors frequently choose inline hooking as opposed to GOT tampering because on hardened Linux systems, the GOT is marked read-only (`RELRO`) after the process is first loaded (see [https://isisblogs.poly.edu/2011/06/01/relro-relocation-read-only](https://isisblogs.poly.edu/2011/06/01/relro-relocation-read-only)).

The `linux_apihooks` plugin detects inline hooks by first gathering the list of functions an application uses in the same manner as `linux_plthook`. It then disassembles the first several instructions of each function, looking for control flow transfers such as `CALL`, `JMP`, or `RET` instructions that point outside the current library.

In the following output, you can see `linux_apihooks` detecting a malware sample that hooks several functions within `libc`:

```
$ python vol.py -f hooks.lime --profile=LinuxDebian-3_2x64 
    linux_apihooks -p 65033
Volatility Foundation Volatility Framework 2.4
Hook VMA                   Hooked Symbol Symbol Address Type  Hook Address
------------------------   ----------    -------------  ----- --------------
/lib/<snip>/libc-2.13.so   write         0x7ff29b512c06 JMP   0x7ff31c069a45
/lib/<snip>/libc-2.13.so   close         0x7ff29b512c46 JMP   0x7ff31c069a96
/lib/<snip>/libc-2.13.so   read          0x7ff29b512c56 JMP   0x7ff31c068e45
/lib/<snip>/libc-2.13.so   open          0x7ff29b512ca6 JMP   0x7ff31c069d13
/lib/<snip>/libc-2.13.so   accept        0x7ff29b512cb6 JMP   0x7ff31c069c57
/lib/<snip>/libc-2.13.so   socket        0x7ff29b512bb0 JMP   0x7ff31c069043
```

Based on the functions hooked (`write`, `close`, `read`, `open`, `accept`, and `socket`), you can surmise that the malware is likely trying to hide file system and network activity. Because file handles and network sockets are both just file descriptors in Linux, hooking the chosen functions can simultaneously control both file system interaction and network connections made by affected userland processes. To further investigate the purpose of the hooks, you can follow the address in the `Hook Address` column in `volshell` or dump the containing memory range (`linux_dump_maps`) to disk for static analysis.

## Summary

Understanding how Linux processes interact with operating system resources (system calls, the file system, network, shared libraries, and so on) is critical to detecting usermode rootkits. Numerous powerful rootkits have been seen in both proof-of-concept and fully weaponized forms—a trend that won’t stop anytime soon. Luckily, memory forensics gives you an equally powerful mechanism to analyze systems for signs of code injection, overwritten GOT entries, inline function hooks, and even the most subtle of environment variable modifications.
