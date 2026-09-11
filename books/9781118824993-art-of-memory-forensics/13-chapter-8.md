# Chapter 8  
 Hunting Malware in Process Memory

The previous chapter introduced you to process memory internals and set the foundations for you to deep dive into analysis. Now you’ll see some specific examples of how you can detect malware that hides in process memory by unlinking dynamic linked libraries (DLLs) or using one of four different methods of injecting code. You’ll also learn the fundamentals of dumping processes, libraries, and kernel modules (any portable executable [PE] files) from memory, including samples that are initially packed or compressed.

## Process Environment Block

Every `_EPROCESS` structure contains a member called the Process Environment Block (PEB). The PEB contains the full path to the process’ executable, the full command line that starts the process, the current working directory, pointers to the process’ heaps, standard handles, and three doubly linked lists that contain the full path to DLLs loaded by the process.

---

## **Analysis Objectives**

Your objectives are these:

- **Recover command lines and process paths:** Learn about sources in process memory that can provide details about how a process was invoked and where its file resides on disk.
- **Analyze heaps:** Learn what types of data applications store on their heap(s) and see a practical example that locates the text typed into Notepad.
- **Inspect environment variables:** Learn how to detect search order hijacking and malware families that mark their presence by creating new variables.
- **Detect backdoors with standard handles:** Determine whether a process’ input and output are being redirected over a remote network socket to an attacker.
- **Enumerate DLLs:** Learn how the operating system tracks DLLs loaded by a process, as well as the APIs that you use on live systems to list them. At the same time, you’ll see how to detect hidden and unlinked libraries.
- **Extract PE files from memory**: Learn how to dump PE files from memory and prepare them for static analysis with a disassembler. You’ll be exposed to the ways PE files change when loaded into memory and how the changes can affect your investigations.
- **Detect code injection**: See detailed descriptions of four types of code injection, including how to detect them with memory forensics.

## **Data Structures**

The main PEB structure is appropriately named `_PEB`. The following code shows how it appears for a 64–bit Windows 7 system, along with the structures for process parameters and DLLs. Remember that the `_PEB` structure exists in process memory, so a process can easily modify its own values to falsely report information or thwart analysis. Later in the chapter, you’ll see how to leverage data in the kernel (such as virtual address descriptors [VADs] and page tables) to cross-reference with some of the information available in the PEB.

```
>>> dt("_PEB")
'_PEB' (896 bytes)
0x0   : InheritedAddressSpace    ['unsigned char']
0x1   : ReadImageFileExecOptions ['unsigned char']
0x2   : BeingDebugged            ['unsigned char']
[snip]
0x10  : ImageBaseAddress         ['pointer64', ['void']]
0x18  : Ldr                      ['pointer64', ['_PEB_LDR_DATA']]
0x20  : ProcessParameters        ['pointer64', ['_RTL_USER_PROCESS_PARAMETERS']]
0x28  : SubSystemData            ['pointer64', ['void']]
0x30  : ProcessHeap              ['pointer64', ['void']]
[snip]
0xe8  : NumberOfHeaps            ['unsigned long']
0xec  : MaximumNumberOfHeaps     ['unsigned long']
0xf0  : ProcessHeaps             ['pointer', ['array', 
 lambda x: x.NumberOfHeaps, ['pointer', ['_HEAP']]]]

>>> dt("_RTL_USER_PROCESS_PARAMETERS")
'_RTL_USER_PROCESS_PARAMETERS' (1024 bytes)
[snip]
0x20  : StandardInput                  ['pointer64', ['void']]
0x28  : StandardOutput                 ['pointer64', ['void']]
0x30  : StandardError                  ['pointer64', ['void']]
0x38  : CurrentDirectory               ['_CURDIR']
0x50  : DllPath                        ['_UNICODE_STRING']
0x60  : ImagePathName                  ['_UNICODE_STRING']
0x70  : CommandLine                    ['_UNICODE_STRING']
0x80  : Environment                    ['pointer64', ['void']]
[snip]

>>> dt("_PEB_LDR_DATA")
'_PEB_LDR_DATA' (88 bytes)
[snip]
0x10  : InLoadOrderModuleList          ['_LIST_ENTRY']
0x20  : InMemoryOrderModuleList        ['_LIST_ENTRY']
0x30  : InInitializationOrderModuleList ['_LIST_ENTRY']
[snip]

>>> dt("_LDR_DATA_TABLE_ENTRY")
'_LDR_DATA_TABLE_ENTRY' (224 bytes)
0x0   : InLoadOrderLinks               ['_LIST_ENTRY']
0x10  : InMemoryOrderLinks             ['_LIST_ENTRY']
0x20  : InInitializationOrderLinks     ['_LIST_ENTRY']
0x30  : DllBase                        ['pointer64', ['void']]
0x38  : EntryPoint                     ['pointer64', ['void']]
0x40  : SizeOfImage                    ['unsigned long']
0x48  : FullDllName                    ['_UNICODE_STRING']
0x58  : BaseDllName                    ['_UNICODE_STRING']
0x68  : Flags                          ['unsigned long']
0x6c  : LoadCount                      ['unsigned short']
[snip]
```

## **Key Points**

The key points for `_PEB` are these:

- `BeingDebugged`: Tells you whether the process is currently being debugged. In the past, we’ve seen malware that attaches to itself (by calling `DebugActiveProcess`). Because only one debugger at a time can attach to a target process, it served as anti-debugging protection. Thus, there is a red flag if this value is set to true, but there are no legitimate active debuggers running.
- `ImageBaseAddress`: The address in process memory where the main executable (`.exe`) is loaded. Before Volatility’s `procdump` plugin (described later in the chapter) carves an executable from memory, it reads this value so it knows where to look.
- `Ldr`: Points to a `_PEB_LDR_DATA` structure, which contains details about the DLLs loaded in a process.
- `ProcessParameters`: Points to a `_RTL_PROCESS_PARAMETERS` structure (described soon).
- `ProcessHeap`: Primary heap for the process, which is created automatically when the process is initialized.
- `NumberOfHeaps`: Number of heaps in a process. By default, a process has only one heap, but it can create others by calling `HeapCreate`.
- `ProcessHeaps`: An array of pointers to process heaps. The first entry in this list always points to the same location as `ProcessHeap` because it is the primary.

Your key points for `_RTL_PROCESS_PARAMETERS` are these:

- `StandardInput`: The process’ standard input handle.
- `StandardOutput`: The process’ standard output handle.
- `StandardError`: The process’ standard error handle.
- `CurrentDirectory`: The current working directory for the application.
- `ImagePathName`: The Unicode full path on disk to the process executable (`.exe`). You often need to consult this value because the `_EPROCESS.ImageFileName` (printed by the `pslist` plugin) contains only the first 16 characters and it does not include Unicode.
- `CommandLine`: The full command line, including all arguments, used to invoke the process.
- `Environment`: A pointer to the process’ environment variables.

Your key points for `_PEB_LDR_DATA` follow. All the linked lists contain elements of type `_LDR_DATA_TABLE_ENTRY`, which is described next. Also, the term “module” here refers to any executable image, which includes the process executable and DLLs.

- `InLoadOrderModuleList`: A linked list that organizes modules in the order in which they are loaded into a process. Because the process executable is always first to load in the process address space, its entry is first in this list.
- `InMemoryOrderModuleList`: A linked list that organizes modules in the order in which they appear in the process’ virtual memory layout. For example, the last DLL to load may end up at a lower base address than the first (due to address space layout randomization [ASLR] and other factors).
- `InInitializationOrderModuleList`: A linked list that organizes modules in the order in which their `DllMain` function was executed. This is different from the load order list because a module’s `DllMain` isn’t always called immediately when it loads. Sometimes it’s never called, for example when you load a DLL as a data file or image resource (see the `dwFlags` parameter to `LoadLibraryEx`).

The key points for `_LDR_DATA_TABLE_ENTRY` are the following:

- DllBase: This is the base address of the module in process memory. The DLL dumping plugins that you’ll learn about later in the chapter will read this address to know where to start carving.
- `EntryPoint`: The first instruction executed by the module. In most cases, it is taken from the PE file’s `AddressOfEntryPoint` value.
- `SizeOfImage`: The size of the module, in bytes.
- `FullDllName`: The full path to the module’s file on disk (for example, `C:\Windows\System32\kernel32.dll`).
- `BaseDllName`: The base portion of the module’s filename (for example, `kernel32.dll`).
- `LoadCount`: The number of times `LoadLibrary` was called for the module. It is used as a reference count to know when it is safe to unload a DLL from process memory. You’ll see this value later in the chapter to determine how a DLL was loaded (via the import address table [IAT] or an explicit call to `LoadLibrary`).

---

### Process Heaps

From a forensics perspective, when you dump process memory via `memdump` or `vaddump`, you inevitably get the heap contents (at least the pages that are not swapped). The same applies to scanning memory with Yara and the `search_process_memory` API discussed in Chapter 7. The problem is that you won’t necessarily know which offsets in your dump file or signature results correspond to heap regions. Furthermore, in some cases you might want to analyze *only* heap memory. For example, say you’re looking for the data an application received over the network or the text a user typed into a word processor. These types of data have a good chance of being on one of the process’ heaps, so there’s no need to waste time scanning memory regions that contain DLLs, stacks, or mapped files.

---

**NOTE**

For a thorough overview of modern heaps structures and internals, see *Windows 8 Heap Internals* by Chris Valasek and Tarjei Mandt: [http://illmatics.com/Windows%208%20Heap%20Internals.pdf](http://illmatics.com/Windows%208%20Heap%20Internals.pdf)

---

### Finding Text on Notepad’s Heap

This example demonstrates how you can drastically narrow the search space when looking for forensic evidence on the heap. The research was sparked when a member of the Volatility user’s mailing list asked this question: How do I find the text a user entered into Notepad? One way, of course, is to reverse engineer `notepad.exe` and determine where it stores the pointer to data that the application receives from the keyboard. However, we took a slightly easier, more black-box approach. First, to set up the environment, we started two instances of Notepad: one opened a rather large log file and the other was used by the “suspect” to develop a plan for committing a crime (the one you’re investigating). An example of the suspect’s desktop is shown in [Figure 8-1](#figure8-1).

![c08f001.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f001.png)

*[**Figure 8-1:**](#figureanchor8-1) The text you see in the open Notepad windows is stored on the Notepad process’ heap*

Suppose that you obtained a memory dump from the suspect’s system. At this time, you can run plugins such as `vadinfo` and `vadtree` on the Notepad processes, but (on this particular machine) you’ll see 50+ VAD nodes that encompass more than 25MB of data in each process. How do you pinpoint completely arbitrary text in a collection of 25 million bytes? You can begin by filtering out only the VADs that contain process heaps.

As the following output from the `heaps` plugins shows, the process with PID 3988 has 6 heaps. Collectively, this narrows your search to 6 VADs and approximately 1.3MB of data. At this point, the black-box approach kicked in. We started looking for anything that might help isolate the heap chunks containing the desired text from all others. As shown, two chunks stood out because the “extra” flag is displayed. In other words, the `HEAP_ENTRY_EXTRA_PRESENT` flag was set in the `_HEAP_ENTRY.Flags` member for these two chunks. Both chunks existed in the primary heap (the first one in the `ProcessHeaps` array) that starts at `0xa0000`:

```
$ python vol.py -f Win2K3SP1x86.vmem --profile=Win2003SP1x86 
    heaps -p 3988 

Volatility Foundation Volatility Framework 2.4
**************************************************
Process: notepad.exe, Pid: 3988
PEB: 0x7ffdf000, Heap: 0xa0000, NumberOfHeaps: 0x6
**********
_HEAP at 0xa0000
  0xa0640 Size: 0x8 Previous: 0xc8 Flags: busy 
  0xa0680 Size: 0x301 Previous: 0x8 Flags: busy 
  0xa1e88 Size: 0x4 Previous: 0x301 Flags: busy 
  0xa1ea8 Size: 0xb Previous: 0x4 Flags: busy 
  0xa1f00 Size: 0xb Previous: 0xb Flags: busy 
  [snip]
  0xa8028 Size: 0x3 Previous: 0x12 Flags: busy, extra
  [snip]
  0xac740 Size: 0xe Previous: 0xb Flags: busy 
  0xac7b0 Size: 0x37 Previous: 0xe Flags: busy, extra
  0xac968 Size: 0xd3 Previous: 0x37 Flags: last 
**********
_HEAP at 0x1a0000
  0x1a0640 Size: 0x8 Previous: 0xc8 Flags: busy 
  0x1a0680 Size: 0x530 Previous: 0x8 Flags: last 
**********
_HEAP at 0x3c0000
  0x3c0640 Size: 0x8 Previous: 0xc8 Flags: busy 
  0x3c0680 Size: 0x301 Previous: 0x8 Flags: busy 
  0x3c1e88 Size: 0x3 Previous: 0x301 Flags: busy 
  0x3c1ea0 Size: 0x2c Previous: 0x3 Flags: last 
[snip]
```

Regardless of the actual meaning of the “extra” flag (we’ve never found a good description) it was a discrepancy that turned our attention to the chunks at `0xa8028` and `0xac7b0`. The following `volshell` command shows that one of these chunks does indeed contain a Unicode version of our suspect’s text. Note that the text actually starts at `0xac7b8` because a `_HEAP_ENTRY` exists at the base, which is 8 bytes on a 32-bit platform.

```
$ python vol.py -f Win2K3SP1x86.vmem --profile=Win2003SP1x86 
     volshell -p 3988

Volatility Foundation Volatility Framework 2.4
Current context: process notepad.exe, pid=3988, ppid=416 DTB=0x140ab4e0
To get help, type 'hh()'
>>> db(0xac7b0)
0x000ac7b0  3700 0e00 e523 1000 4c00 6900 7300 7400   7....#..L.i.s.t.
0x000ac7c0  2000 6f00 6600 2000 7400 6100 7200 6700   ..o.f...t.a.r.g.
0x000ac7d0  6500 7400 7300 3a00 0d00 0a00 0d00 0a00   e.t.s.:.........
0x000ac7e0  4a00 6900 6d00 2000 4a00 6100 6d00 6500   J.i.m...J.a.m.e.
0x000ac7f0  7300 0d00 0a00 4200 6f00 6200 6200 7900   s.....B.o.b.b.y.
0x000ac800  2000 4b00 6e00 6900 6700 6800 7400 2000   ..K.n.i.g.h.t...
0x000ac810  0d00 0a00 5000 6500 7400 6500 7200 2000   ....P.e.t.e.r...
0x000ac820  5300 6900 6c00 7600 6500 7200 0d00 0a00   S.i.l.v.e.r.....
```

At this point, we built a `notepad` plugin that automatically dumps the text from `notepad.exe` processes. Here’s how the plugin’s output looks:

```
$ python vol.py –f Win2K3SP1x86.vmem --profile=Win2003SP1x86 notepad
Volatility Foundation Volatility Framework 2.4

Process: 3988
Text:
List of targets:
Jim James
Bobby Knight 
Peter Silver
Amy Christoph
Plan: 
Get their cell phone numbers
Text with a place to meet
Blackmail with pictures 
Collect money and profit
```

Despite being entirely based on an educated guess rather than empirical testing or reverse engineering, the plugin works consistently and accurately on 32-bit versions of XP and 2003 Server. Even if we didn’t catch a break with the “extra” flag indicators, it is still possible to narrow the search space to 1.3 MB out of the entire process memory space by just looking at the regions that contain heaps.

### Environment Variables

A process’ environment variables are pointed to by `_PEB.ProcessParameters.Environment`. The variables are organized as multiple NULL-terminated strings, similar to a `REG_MULTI_SZ` value in the registry. If an attacker manipulates these variables, they can cause the target application to unexpectedly execute a malicious process. Additionally, some malware marks its presence by creating environment variables rather than mutexes. (see the “Coreflood Presence Marking” section) Thus, you should know how to check for suspicious entries. [Table 8-1](#table8-1) categorizes the different types of variables according to their scope and persistence.

*[**Table 8-1**](#tableanchor8-1): Sources and Scopes of Environment Variables*

|  |  |  |  |
| --- | --- | --- | --- |
| **Type** | **Source** | **Scope** | **Persists** |
| System | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` | Everyone (all processes) | Yes |
| User | `HCKU\Environment` | A user | Yes |
| Volatile | `HKCU\Volatile Environment` | A user | No |
| Dynamic | `SetEnvironmentVariable` | A process | No |

The *System* and *User* variables are both persistent in the registry. Thus, you can enumerate them by parsing registry hive files that were acquired from disk. The *Volatile* variables are also stored in the registry, but in a volatile key, so you must access them by capturing RAM and using Volatility’s cached registry support (see Chapter 10). The *Dynamic* variables are set per process when a thread within the process calls `SetEnvironmentVariable`. Similar to the Volatile variables, these entries are found only in memory—they’re never written to the registry hives on disk or any other log file.

When a process is created, it usually inherits the environment block from its parent. The parent process can override this default behavior by specifying the `lpEnvironment` parameter when it calls `CreateProcess`. Here’s a list of the types of data you can typically find in environment variables:

- Paths to executable programs (`PATH`)
- Extensions assigned to executable programs (`PATHEXT`)
- Paths to temporary directories
- Paths to a user’s Documents, Internet History, and Application Data folders
- User names, computer names, and domain names
- The location of `cmd.exe` (`ComSpec`)

#### Attacks on Environment Variables

The two most common types of attacks on environment variables include changing the `PATH` and `PATHEXT` variables. Modifying these values has an effect similar effect to search-order hijacking. For example, consider the following scenario:

```
PATH=C:\windows;C:\windows\system32
PATH=C:\Users\HR101\.tmp;C:\windows;C:\windows\system32
```

In this case, if the `PATH` variable is updated in `explorer.exe` and the logged-on user goes to Start ⇒ Run and types “**calc**”, the system will search for an application named “calc” in the `C:\users\HR101\.tmp` directory before it looks in the `windows` and `system32` directories. Thus, the user will unexpectedly execute malicious code. Likewise, the `PATHEXT` variable contains a list of extensions that are searched if a user fails to specify one. For example, if you type “**calc**” as previously mentioned, the system will look for `calc.com` first, then `calc.exe`, then `calc.bat` , and so on. Due to the following modifications, an attacker could plant a file named `calc.zzz` in one of the searched directories, and it would be executed first:

```
PATHEXT=.COM;.EXE;.BAT;.CMD;.VBS;.VBE
PATHEXT=.ZZZ;.COM;.EXE;.BAT;.CMD;.VBS;.VBE
```

Of course, in these situations, the malicious code being invoked would subsequently launch the legitimate `calc.exe`—so the user doesn’t start to suspect foul play.

#### Coreflood Presence Marking

Many malware samples mark their presence on a system by creating globally accessible mutexes. This helps prevent accidental re-infection. Mutexes also provide a strong forensic indicator. Coreflood used environment variables for a very similar purpose—to mark its presence within a process. Because Coreflood’s main component is a DLL, the authors needed a way to make sure that the same process didn’t get injected with the malware more than once. Thus, they programmed the DLL’s entry function to create a pseudo-randomly generated string based on the parent process’ PID and the disk’s volume serial number. The string was then added to the process’ environment variables by calling `SetEnvironmentVariable`, if it didn’t already exist. Otherwise, the DLL unloaded immediately. A graph of the DLL’s entry function (produced by IDA Pro) is shown in [Figure 8-2](#figure8-2).

![c08f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f002.png)

*[**Figure 8-2:**](#figureanchor8-2) A graph showing the functions called by Coreflood’s entry point*

An interesting riddle that we like to ask students in our training class is this: Based on what you just learned, how many of the following processes are infected with Coreflood’s DLL? Note that we are using Volatility’s `envars` plugin to enumerate the variables in a memory dump.

```
$ python vol.py -f coreflood.img --profile=WinXPSP3x86 envars

**************************************************
Pid 1144 explorer.exe (PPid 644) Block at 0x10000
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
windir=C:\WINDOWS
[SNIP]

**************************************************
Pid 512 IEXPLORE.EXE (PPid 1144) Block at 0x10000
ProgramFiles=C:\Program Files
QBYXKDAGXM=EWONSYG
SESSIONNAME=Console
USERDOMAIN=JAN-DF663B3DBF1
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
[SNIP]

**************************************************
Pid 560 notepad.exe (PPid 1144) Block at 0x10000
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
[SNIP]

**************************************************
Pid 220 firefox.exe (PPid 1144) Block at 0x10000
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
[SNIP]
```

If you answered “two,” you are correct. At first glance, it appears as if all four processes are infected, because they all contain suspicious variable names. However, we previously mentioned that child processes typically inherit their parent’s variables. We also said the variable name depends on the PID of the process. If all four processes were infected, you’d see four unique variable names; but you see only two (`VFTPXXPYVTAMF` and `QBYXKDAGXM`). The last three processes (`IEXPLORE.EXE`, `notepad.exe`, and `firefox.exe`) were spawned by PID 1144, which explains why they have a copy of the `VFTPXXPYVTAMF` variable. You’ll notice that `IEXPLORE.EXE` actually has two variables: the one it inherited from its parent and the one Coreflood calculated for its own PID.

---

**NOTE**

The `--silent` option to the `envars` plugin causes it to suppress known variables (based on a hard-coded whitelist). This is handy in situations where a variable’s name or value isn’t blatantly obvious; for example, like `VFTPXXPYVTAMF`.

---

### Standard Handles

By analyzing a process’ standard handles, you can determine where it gets input and where it sends output and error messages. This is especially useful when investigating potential breaches by remote attackers. For example, a fairly common way to create a backdoor command shell on a system involves spawning an instance of `cmd.exe` and redirecting its standard handles over named pipes or network sockets. Thus, the attackers can use `telnet` or `netcat` to connect to the target machine (provided that no firewall is blocking access), and type commands as if they were sitting at the console. The following code shows the relevant C source code for a backdoor that uses this technique:

```
  1 mySockAddr.sin_family = PF_INET;
  2 mySockAddr.sin_port = htons(31337);
  3 mySockAddr.sin_addr.s_addr = htonl(INADDR_ANY);
  4 
  5 if (bind(myMainSock, (SOCKADDR *)&mySockAddr, 
  6     sizeof(mySockAddr)) == SOCKET_ERROR)
  7 {   
  8     WSACleanup();
  9     return(-1);
 10 }
 11 
 12 while(1) {
 13     myCliSock = SOCKET_ERROR;
 14     while(myCliSock == SOCKET_ERROR) {
 15         Sleep(1);
 16         listen(myMainSock, SOMAXCONN);
 17         ulSzSockAddr = sizeof(myCliSockAddr);
 18         myCliSock = accept(myMainSock, 
 19             (SOCKADDR *)&myCliSockAddr, 
 20             (int *)&ulSzSockAddr);
 21     }
 22     memset(&mySi, 0, sizeof(mySi));
 23     memset(&myPi, 0, sizeof(myPi));
 24     mySi.wShowWindow = SW_HIDE;
 25     mySi.dwFlags = STARTF_USESTDHANDLES | 
 26                    STARTF_USESHOWWINDOW;
 27     mySi.hStdError = (VOID *)myCliSock;
 28     mySi.hStdInput = (VOID *)myCliSock;
 29     mySi.hStdOutput = (VOID *)myCliSock;
 30     CreateProcess(0, wcsdup(L"cmd.exe"), 0, 0, 
 31                   1, // bInheritHandles = TRUE
 32                   0, 0, 0, &mySi, &myPi);
 33 }
```

Lines 1–3 configure an IPv4 socket that will listen on all interfaces (`INADDR_ANY`) on port 31337. Lines 5–10 bind the local address to the socket. The function then enters a while loop (lines 12–33) that executes for as long as the process stays running. On lines 16–20, the program begins to listen on the port and calls `accept` to enter a state in which it can receive client connections. After an incoming connection is received, it sets up the specifications for the child `cmd.exe` process on line 25. In particular, it enables `STARTF_USESTDHANDLES` in the `_STANDARD_INFORMATION` block (`mySi`) and then sets the `hStdError`, `hStdInput`, and `hStdOutput` values to the client socket (`myCliSock`). Finally, on lines 30–32, the `cmd.exe` instance is started. Note that `bInheritHandles` is set to 1, which means that the child process inherits a copy of the parent’s handles (specifically for access to the `myCliSock` socket handle).

At this point, any commands that the attackers type over the network are tunneled straight into `cmd.exe` via its standard input. Results are passed back to the client over standard output. If you’re investigating the victim machine and see `cmd.exe` running, you might not think anything of it. However, you’d be overlooking the one piece of evidence that matters most. Take a look at how the standard handles appear for the process that is involved in the backdoor activity:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 volshell
Volatility Foundation Volatility Framework 2.4
Current context: process System, pid=4, ppid=0 DTB=0x187000
To get help, type 'hh()'
>>> for proc in win32.tasks.pslist(self.addrspace):
...   if str(proc.ImageFileName) != "cmd.exe":
...     continue
...   if proc.Peb:
...     print proc.UniqueProcessId,\
...     hex(proc.Peb.ProcessParameters.StandardInput),\
...     hex(proc.Peb.ProcessParameters.StandardOutput),\
...     hex(proc.Peb.ProcessParameters.StandardError)
... 
572 0x3L 0x7L 0xbL
3436 0x3L 0x7L 0xbL
564 0x3L 0x7L 0xbL
2160 0x68L 0x68L 0x68L
```

Four instances of `cmd.exe` are running on the machine. The first three have normal values for standard input (`0x3`), standard output (`0x7`), and standard error (`0xb`). The last one, PID 2160, displays `0x68` for all handles. You can determine whether `0x68` corresponds to a named pipe or network socket by using the `handles` plugin, as shown in the following output. In Chapter 11, you’ll learn that open handles to `\Device\Afd\Endpoint` indicate network activity (`Afd` is the Auxiliary Function Driver for Winsock).

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 handles -p 2160 -t File 
Volatility Foundation Volatility Framework 2.4
Offset(V)             Pid     Handle  Type    Details
------------------ ------ ---------  -------- -------
0xfffffa80015c4070   2160       0xc  File      
\Device\HarddiskVolume1\Users\Elliot\Desktop
0xfffffa8002842130   2160      0x54  File      \Device\Afd\Endpoint
0xfffffa80014f3af0   2160      0x68  File      \Device\Afd\Endpoint
```

Although `cmd.exe` didn’t create any sockets, it has access to them as a result of the inheritance. You can determine the exact port the backdoor uses, as well as the remote endpoint if any connections are currently established, by first looking up the parent process for `cmd.exe` and then listing its network information (see Chapter 11). You can complete these two actions with the `pstree` and `netscan` plugins, as shown in the following output:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 pstree
Volatility Foundation Volatility Framework 2.4
Name                                   Pid   PPid   Thds   Hnds
------------------------------------ ------ ------ ------ ------ 
[snip]
.. 0xfffffa80011105e0:memen.exe        1400    572      1     30 
... 0xfffffa8002b42060:cmd.exe         2160   1400      1     25 
. 0xfffffa8002827060:cmd.exe           3436   1408      1     25 
 0xfffffa8002be6b30:moby.exe           3036   3024     15    385 

$ python vol.py -f memory.dmp --profile=Win7SP1x64 netscan 
Volatility Foundation Volatility Framework 2.4 
Proto    Local Address          Foreign Address      State        Pid  Owner    
TCPv4    0.0.0.0:31337          0.0.0.0:0            LISTENING    1400 memen.exe
TCPv4    192.168.228.171:31337  <REDACTED>:59574     ESTABLISHED  1400 memen.exe
```

The parent of `cmd.exe` PID 2160 is `memen.exe` PID 1400. The parent has one listening IPv4 socket on port 31337 and one established connection (the remote endpoint is redacted). This is an interesting example because the primary network activity maps back to the parent process, but the child is the one actually being used to execute commands and pass the results back to the attacker.

### Dynamic Link Libraries (DLLs)

DLLs contain code and resources that can be shared between multiple processes. They’re popular among malware and threat actors because DLLs are designed to run inside a host process, thus giving them access to all of the process’ resources: its threads, handles, and full range of process memory. Furthermore, DLLs allow toolkits to be modular and extensible. At the very least, when analyzing DLLs, you should check for the following:

- **List discrepancies:** As you’ll learn soon, adversaries try to hide their DLLs by unlinking metadata structures from one or more lists or by overwriting the name or path fields in the metadata structures. You can detect such attempts by cross-referencing.
- **Unexpected name paths:** Be aware of suspiciously named DLLs (such as `sodapop.dll`) as well as familiar-looking names in nonstandard locations (i.e., `C:\Windows\system32\``sys``\kernel32.dll`). We have also seen malware loading DLLs from sectors outside the NTFS partition. For example, a TDL variant referenced a module whose path started with `\\.\globalroot\Device\svchost.exe`.
- **Context:** DLLs such as `ws2_32.dll`, `crypt32.dll`, `hnetcfg.dll`, and `pstorec.dll` are used for networking, cryptography, firewall maintenance, and access to protected storage, respectively. They aren’t suspicious *per se*, but you have to consider the purpose of the application in which they are loaded.

#### How DLLs Are Loaded

DLLs can be loaded in the following ways:

- **Dynamic linking:** As part of the process initialization routines, any DLL in the executable (`.exe`) file’s IAT automatically loads in the process’ address space.
- **Dependencies:** DLLs also have import tables, so when they’re loaded, all additional DLLs on which they rely load into the process’ address space. For more information, see Dependency Walker ([http://www.dependencywalker.com/](http://www.dependencywalker.com/)).
- **Run-time dynamic linking (RTDL):** A thread can explicitly call `LoadLibrary` (or the native `LdrLoadDll`) with the name of the DLL to load. This has the same end result as dynamic linking (a DLL loaded in the process), except there’s no trace of the DLL in the process’ IAT.
- **Injections:** As you’ll learn later in the chapter, DLLs can also be forcefully injected into a target process.

#### Enumerating DLLs on Live Systems

Your basic system investigation tools will have the capability to list DLLs. For example, Process Hacker and Process Explorer both support it. Sysinternals also provides a command-line utility called `listdlls` ([http://technet.microsoft.com/en-us/sysinternals/bb896656.aspx](http://technet.microsoft.com/en-us/sysinternals/bb896656.aspx)). From a programming perspective, you can leverage the Windows API functions `CreateToolhelp32Snapshot` along with `Module32First` and `Module32Next`. Alternately, another function, `EnumProcessModules` has been available since Windows XP.

The important part about these tools and the APIs on which they depend is where they get the information. As presented in the data structures section of this chapter, three linked lists of DLLs are accessible from the PEB. They store metadata about modules in the order they were loaded and initialized, as well as where they exist in memory. Live APIs and tools typically only look at the load order list. This presents attackers the opportunity to manipulate and hide DLLs.

#### Hiding DLLs

As shown in [Figure 8-3](#figure8-3), because all three lists exist in process memory, any thread running in the process can unlink a metadata structure (`_LDR_DATA_TABLE_ENTRY`) to hide it from the running system (and potentially memory forensics as well). For example, once loaded, the `xyz.dll` module can overwrite its own `Flink` and `Blink` pointers so that its entry is skipped during enumeration.

![c08f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f003.png)

*[**Figure 8-3:**](#figureanchor8-3) A diagram showing how the PEB points to three doubly linked lists of DLLs*

#### Listing DLLs in Memory

The unlinking approach depicted in [Figure 8-3](#figure8-3) also affects Volatility’s `dlllist` plugin. We designed `dlllist` like this for a reason. It enumerates DLLs by walking the load order list, just like live APIs, because in some cases it’s good to see things from the perspective of tools that run on the live system. Here’s an example output from `dlllist`. In our training class, we typically ask students if they can spot anything suspicious. Can you?

```
$ python vol.py –f mem.dmp --profile=WinXPSP3x86 dlllist –p 3108

notepad.exe pid:   3108
Command line : "C:\WINDOWS\system32\notepad.exe" 
Service Pack 3

Base             Size  LoadCount Path
---------- ---------- ---------- ----
0x01000000    0x14000     0xffff C:\WINDOWS\system32\notepad.exe
0x7c900000    0xb2000     0xffff C:\WINDOWS\system32\ntdll.dll
0x7c800000    0xc60f000     0xffff C:\WINDOWS\system32\kernel32.dll
0x77dd0000    0x9b000     0xffff C:\WINDOWS\system32\ADVAPI32.dll
0x77fc00e000    0x11000     0xffff C:\WINDOWS\system32\Secur32.dll
0x77c10000    0x58000     0xffff C:\WINDOWS\system32\msvcrt.dll
0x77f10000    0x49000     0xffff C:\WINDOWS\system32\GDI32.dll
0x7e410000    0x91000     0xffff C:\WINDOWS\system32\USER32.dll
0x7c9c0000   0x817000     0xffff C:\WINDOWS\system32\SHELL32.dll
<snip>
0x7e1c00e000    0xa2000        0x3 C:\WINDOWS\system32\urlmon.dll
0x771b0000    0xaa000        0x3 C:\WINDOWS\system32\WININET.dll
0x77a80000    0x95000        0x3 C:\WINDOWS\system32\CRYPT32.dll
0x71ab0000    0x17000       0x27 C:\WINDOWS\system32\WS2_32.dll
0x71a50000    0x3f000        0x4 C:\WINDOWS\system32\mswsock.dll
0x662b0000    0x58000        0x1 C:\WINDOWS\system32\hnetcfg.dll
0x76f20000    0x27000        0x1 C:\WINDOWS\system32\DNSAPI.dll
```

The process executable (`notepad.exe`) is first to load in the process address space, thus it’s first in the load order list. Next, the `ntdll.dll` and `kernel32.dll` system libraries are loaded. The system then proceeds to load any DLLs in Notepad’s IAT and any dependency modules that those DLLs need. Note, however, that the load count for the first batch of modules is `0xffff`. Because this field is a short integer, `0xffff` is actually `-1`. A load count of `-1` means the DLL loaded because it was specified in an IAT. The others near the end, whose load counts are `0x3`, `0x27`, `0x4` and `0x1`, were all loaded via explicit calls to `LoadLibrary`.

Although there are plenty of legitimate reasons to call `LoadLibrary`, its usage is also consistent with the techniques shellcode uses to set up the target process’ address space. You may notice none of the explicitly loaded DLLs are suspicious per se—they’re all properly named and in the correct `system32` path. However, when you consider their purpose (network related) and the host process context (`notepad.exe`), the situation begins to look quite abnormal. Most likely code needing access to networking DLLs has infected this process, and it loads them by calling `LoadLibrary`.

---

**NOTE**

The DLL layout is slightly different for WOW64 processes (32-bit applications on a 64–bit operating system). The three lists in the PEB only contain the DLLs that can be accessed by the 32-bit application (i.e. those below `MmHighestUserAddress`) and the WOW64 compatibility DLLs. For example, `dlllist` output for these processes looks like this:

```
$ python vol.py -f win764bit.raw --profile=Win7SP0x64 dlllist -p 2328
Volatility Foundation Volatility Framework 2.4
************************************************************************
iexplore.exe pid:   2328
Command line : "C:\Program Files (x86)\Internet Explorer\iexplore.exe" 
Note: use ldrmodules for listing DLLs in WOW64 processes

Base               Size      LoadCount Path
------------------ --------- ------------------ ----
0x0000000001350000   0xa6000  0xffff C:\Program Files (x86)\Internet 
Explorer\iexplore.exe
0x0000000077520000  0x1ab000  0xffff C:\Windows\SYSTEM32\ntdll.dll
0x0000000073b80000   0x3f000     0x3 C:\Windows\SYSTEM32\wow64.dll
0x0000000073b20000   0x5c000     0x1 C:\Windows\SYSTEM32\wow64win.dll
0x0000000074e10000    0x8000     0x1 C:\Windows\SYSTEM32\wow64cpu.dll
```

The three WOW64 libraries (and `ntdll.dll`) contain the entry points for 32-bit applications and the necessary code to transition into 64-bit mode so the calling thread can access DLLs in higher address regions. As the note tells you, you can use the `ldrmodules` plugin (discussed in detail next) to enumerate DLLs in a WOW64 process—even those not typically accessible to 32-bit applications.

For more information on these concepts, see *WOW64 Implementation Details*: [http://msdn.microsoft.com/en-us/library/windows/desktop/aa384274(v=vs.85).aspx](http://msdn.microsoft.com/en-us/library/windows/desktop/aa384274(v=vs.85).aspx).

---

#### Detecting Unlinked DLLs

The hiding technique shown in [Figure 8-3](#figure8-3) is relatively effective, despite being primitive and very simple. There are more robust ways to hide DLLs that you’ll also encounter in the wild. For example, if there are three lists, and malware hides from one, you can easily cross-reference with the other two and see what’s missing. Thus, as far back as 2007 (see [http://www.openrce.org/blog/view/844/How_to_hide_dll](http://www.openrce.org/blog/view/844/How_to_hide_dll)), folks in the offensive community began unlinking DLLs from all three lists. All attackers had to do was include two to three extra lines of code that cut a metadata structure from the memory order and initialization order lists as well.

Two methods can help you detect DLLs that are unlinked from all three lists:

- **PE file scanning**: You can leverage techniques described in Chapter 7 to perform a brute force scan through process memory, looking for all instances of PE files (based on their known `MZ` header signatures). Remember, however, that the PE headers are also in process memory, so the same body of code that unlinks the DLL metadata structure can easily overwrite them.
- **VAD cross-referencing**: This is the technique implemented by Volatility’s `ldrmodules` plugin. If you recall from Chapter 7, VAD nodes contain full paths on disk to files mapped into the regions—including DLL files. The unique aspect of VADs is that they exist in kernel memory and attempts to manipulate the tree (i.e., unlink a VAD node) or overwrite their file pointers quickly result in system instability (blue screens).

The `ldrmodules` plugin first enumerates all VAD nodes that contain mapped executable images. Specifically, it looks for large nodes with `PAGE_EXECUTE_WRITECOPY` protections, a `VadImageMap` type, and the `Image` control flag set. It then compares the starting addresses from the VAD nodes with the `DllBase` value from the `_LDR_DATA_TABLE_ENTRY` structures found in process memory. Entries identified through the VAD that aren’t represented in the DLL lists are potentially hidden. An example of the output from an infected 64–bit Windows 7 sample follows:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 ldrmodules -p 616 
Volatility Foundation Volatility Framework 2.4 
Process      Base               InLoad InInit InMem MappedPath
------------ ------------------ ------ ------ ----- ----------
svchost.exe  0x0000000074340000 True   True   True  \Windows\[snip]\sfc.dll
svchost.exe  0x00000000779a0000 True   True   True  \Windows\[snip]\ntdll.dll
svchost.exe  0x000007feff570000 False  False  False \Windows\[snip]\lpkz2.dll
svchost.exe  0x0000000077780000 True   True   True  \Windows\[snip]\kernel32.dll
svchost.exe  0x000007fefd990000 True   True   True  \Windows\[snip]\msasn1.dll
svchost.exe  0x000007fefbbc00e000 True   True   True  \Windows\[snip]\wtsapi32.dll
svchost.exe  0x000007fefdac0000 True   True   True  \Windows\[snip]\KernelBase.dll
svchost.exe  0x000007fefcc00000 True   True   True  \Windows\[snip]\gpapi.dll
svchost.exe  0x000007fefb800000 True   True   True  \Windows\[snip]\ntmarta.dll
svchost.exe  0x000007fefcc20000 True   True   True  \Windows\[snip]\userenv.dll
svchost.exe  0x000007fefbd60000 True   True   True  \Windows\[snip]\xmllite.dll
svchost.exe  0x000007feff460000 True   True   True  \Windows\[snip]\oleaut32.dll
svchost.exe  0x000007fefde70000 True   True   True  \Windows\[snip]\urlmon.dll
svchost.exe  0x000007fef9290000 True   True   True  \Windows\[snip]\wscapi.dll
[snip]
svchost.exe  0x00000000ff720000 True   False  True  \Windows\[snip]\svchost.exe
svchost.exe  0x000007fefd8c00f000 True   True   True  \Windows\[snip]\profapi.dll
svchost.exe  0x000007fefdd10000 True   True   True  \Windows\[snip]\usp10.dll
```

`lpkz2.dll` does not exist in any of the three DLL lists in process memory, but the VAD in kernel memory has a record of it. You can also see its base address is `0x000007feff570000`, which you can then pass to the `dlldump` plugin (introduced later in the chapter) to extract it from memory.

Notice that the `svchost.exe` entry displays `False` in the `InInit` column (initialization order list). This is an exception to the rule regarding the ability to detect unlinked DLLs by cross-referencing data sources. You will never find the process executable (`.exe`) in the initialization order list because the executable is initialized differently from all other modules. In particular, it’s not a DLL, so there’s no `DllMain` function to be called.

---

**Warning**

Malware has also been known to overwrite the `FullDllName` and `BaseDllName` members of the `_LDR_DATA_TABLE_ENTRY` structures, rather than unlinking the metadata structure. McAfee reported a ZeroAccess variant behaving in this manner (see *ZeroAccess Misleads Memory-File Link:*[http://blogs.mcafee.com/mcafee-labs/zeroaccess-misleads-memory-file-link](http://blogs.mcafee.com/mcafee-labs/zeroaccess-misleads-memory-file-link)). In this case, live APIs on the system, in addition to Volatility’s `dlllist` plugin, would show a fake path of `c:\windows\system32\n`, but the VAD in kernel memory would still contain the original path to the DLL. The `-v/--verbose` option to `ldrmodules` will print the full paths from all sources, and you’ll see the discrepancy.

---

## PE Files in Memory

One of the most useful features of Volatility is its capability to dump and rebuild PE files (executables, DLLs, and kernel drivers). Because of changes that occur during program execution, it is not likely that you will get an exact copy of the original binary—or even one that runs on another machine. However, the dumped copy should be close enough to the original so that you can disassemble the malware and determine its capabilities, reverse any algorithms, and so forth. We are frequently asked whether it is possible to dump an executable and compare its MD5 or SHA1 hash to the file on disk. Although it may be possible to compare *fuzzy hashes* (e.g., the percentage similarity), cryptographic hashes will never match because of the following:

- **IAT patching:** The loader modifies a PE file’s IAT to contain the addresses of API functions in process memory. The addresses are specific to the machine and instance of the process from which the PE was extracted.
- **Inaccessible sections:** Not all PE sections are loaded into memory. For example, in some cases, the resource section (`.rsrc`) might not be loaded until it’s needed. Obviously, data never read into memory in the first place is not accessible when you dump it.
- **Global variables:** A PE file can define global variables or values in its read/write data section that are modified during execution. Thus, when you dump the PE from memory, you get the current values, not the uninitialized original ones.
- **Self-modifying code:** Many PE files, especially malicious ones, modify their own code at run time. For example, packed applications decompress and/or decrypt instructions in memory.

Although you should be aware of these concepts, they’re not necessarily disadvantages or caveats of memory analysis. In fact, some of them can actually play in your favor. For example, one of the global variables may be an encryption key for command and control traffic. Likewise, the PE file may be packed on disk, preventing you from statically analyzing it. By dumping the PE file from memory, you can recover data in the format that most closely resembles the current state of the application.

---

**NOTE**

We don’t cover the internals of PE files in this book, but it’s very useful to know. For a crash course, see the following classics by Matt Pietrek:

*An In-Depth Look into the Windows Portable Executable File Format*: [http://msdn.microsoft.com/en-us/magazine/cc301805.aspx](http://msdn.microsoft.com/en-us/magazine/cc301805.aspx).

*Peering Inside the PE: A Tour of the Win32 Portable Executable File Format*: [http://msdn.microsoft.com/en-us/magazine/ms809762.aspx](http://msdn.microsoft.com/en-us/magazine/ms809762.aspx).

---

### PE File Slack Space

Another reason why PE files dumped from memory often differ from the original file on disk is because of slack space. The smallest page size on a typical x86 or x64 Windows system is 4,096 bytes. Most PE files have sections that are not exact multiples of the smallest page size. [Figure 8-4](#figure8-4) shows the effect that this has on reconstructing binaries from memory. The `.text` section, which is not an exact multiple of 4,096, must fully exist in memory marked as RX (read, execute), and the `.data` section must fully exist in memory marked as RW (read, write). Because protections are applied at the page level (in other words, if a page is marked as executable, *all* bytes in the page are executable), the two sections must be separated after being loaded into memory. Otherwise, the beginning of the `.data` section ends up being RX instead of RW.

![c08f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f004.png)

*[**Figure 8-4:**](#figureanchor8-4) A diagram showing how PE files expand in memory, leaving slack space between sections*

The dotted lines in [Figure 8-4](#figure8-4) indicate page boundaries, and the filled-in areas represent slack space due to section sizes that are not multiples of the page size. In some cases, the slack space contains just uninitialized data, making it irrelevant to your investigation. However, the slack space can also contain critical evidence, especially when dealing with packed files whose sections shift and change during decompression.

### Parsing PE Headers in Memory

Volatility provides a few APIs that can help you parse PE headers in memory. Plugins exist that wrap most of this functionality, so you don’t need to perform these manual steps. However, for those of you interested in how the PE-dumping plugins work behind the scenes, and especially if you plan on developing your own tools that rely on similar functionality, the following tutorial will be useful. For example, a while back, we wrote a custom plugin to find and automatically extract ZeroAccess binaries ([http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html](http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html)) using the PE dumping APIs.

The following example shows you how to use `volshell` to verify that the process’ claimed base address does in fact contain the signature of a DOS header:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 volshell -p 516
Volatility Foundation Volatility Framework 2.4 
Current context: process lsass.exe, pid=516, ppid=400 DTB=0x19e6b000
To get help, type 'hh()'
>>> process = proc() 
>>> db(process.Peb.ImageBaseAddress)
0xff080000  4d5a 9000 0300 0000 0400 0000 ffff 0000   MZ..............
0xff080010  b800 0000 0000 0000 4000 0000 0000 0000   ........@.......
0xff080020  0000 0000 0000 0000 0000 0000 0000 0000   ................
0xff080030  0000 0000 0000 0000 0000 0000 f000 0000   ................
0xff080040  0e1f ba0e 00b4 09cd 21b8 014c cd21 5468   ........!..L.!Th
0xff080050  6973 2070 726f 6772 616d 2063 616e 6e6f   is.program.canno
0xff080060  7420 6265 2072 756e 2069 6e20 444f 5320   t.be.run.in.DOS.
0xff080070  6d6f 6465 2e0d 0d0a 2400 0000 0000 0000   mode....$.......
```

The `MZ` signature appears intact. The next step to validate the PE header format is to follow the `e_lfanew` member to find the NT header. Instead of doing this by hand, you can create an `_IMAGE_DOS_HEADER` at the base address and then use the `get_nt_header` function. Here’s an example:

```
>>> process_space = process.get_process_address_space()
>>> dos_header = obj.Object("_IMAGE_DOS_HEADER", 
...                   offset = process.Peb.ImageBaseAddress, 
...                   vm = process_space)
...
>>> nt_header = dos_header.get_nt_header()
>>> db(nt_header.obj_offset)
0xfc08f000f0  5045 0000 6486 0600 55c1 5b4a 0000 0000   PE..d...U.[J....
0xff080100  0000 0000 f000 2200 0b02 0900 0028 0000   ......"......(..
0xff080110  0052 0000 0000 0000 5018 0000 0010 0000   .R......P.......
0xff080120  0000 08ff 0000 0000 0010 0000 0002 0000   ................
```

Now that you have an NT header, the following code shows how to print the name, relative virtual address, and virtual size of each section. To compute the absolute address of each section, just add the base address of the DOS header to the relative offset. For example, you can find the `.text` section at `0xff080000 + 0x1000 = 0xff081000` for this process:

```
>>> for sec in nt_header.get_sections():
...   print sec.Name, hex(sec.VirtualAddress), hex(sec.Misc.VirtualSize)
... 
.text   0x1000L 0x26beL
.rdata  0x4000L 0x3a74L
.data   0x8000L 0x7a0L
.pdata  0x9000L 0x3e4L
.rsrc   0xa000L 0x700L
.reloc  0xb000L 0x1d4L
```

You should have a good idea of how the PE file contents are carved from memory at this point. Nevertheless, to complete our example, here’s the code you can use to dump a copy of each section to disk. You can optionally pass a parameter to `get_image` so it preserves slack space; however, you will read more about that in the next section. Note that after we finish writing data to the file (`dumped.exe`), we quit `volshell` and check the output file’s type:

```
>>> outfile = open("dumped.exe", "wb")
>>> for offset, code in dos_header.get_image():
...     outfile.seek(offset)
...     outfile.write(code)
... 
>>> outfile.close()
>>> quit()

$ file dumped.exe 
dumped.exe: PE32+ executable for MS Windows (GUI) 
```

---

**NOTE**

After you create the `_IMAGE_DOS_HEADER` object in Volatility, you can also list its imported or exported functions, print its version information, or perform a number of other actions on a PE file. We don’t show code examples here, but you can see the `pe_vtypes.py` file in the source for API usage.

---

### PE Extraction Plugins

You’ve already seen the APIs and background for how Volatility extracts PE files from memory. This section will show you some of the existing plugins (and the options they take) that can automate the process. Here are their names and short descriptions:

- `procdump`: Dump a process executable. You can identify the process by PID (`--pid`) or the physical offset of its `_EPROCESS` (`--offset`). The latter option enables you to dump processes hidden from the active process list.
- `dlldump`: Dump a DLL. You can identify the *host* process by PID (`--pid`) or the physical offset of its `_EPROCESS` (`--offset`). If the DLL(s) are in the load order list, you can identify them using a regular expression (`--regex/--ignore-case`) on their name. Otherwise, you can refer to them by their base address in process memory (`--base`). The latter option enables you to dump hidden or injected PE files.
- `moddump`: Dump a kernel module. Similar to `dlldump`, if the modules you want are in the loaded modules list (see Chapter 13), you can identify them with regular expressions. Otherwise, to dump a PE file from anywhere in kernel memory, use the `--base` parameter.

All the plugins require an output directory (`--dump-dir`) to write the extracted files. They also all accept an optional `--memory` parameter, which is how you request the slack space between sections to be included in the output file. Thus, when you use `--memory`, the dumped file more closely resembles the PE as it existed in memory. A few examples of using these plugins follow. The first one shows how to extract all executables in the active process list:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
    procdump --dump-dir=OUTDIR/

Volatility Foundation Volatility Framework 2.4
ImageBase          Name                 Result
------------------ -------------------- ------
------------------ System               Error: PEB at 0x0 is unavailable
0x0000000047c50000 smss.exe             OK: executable.256.exe
0x000000004a3c00e000 csrss.exe            OK: executable.348.exe
0x00000000ff9c0000 wininit.exe          OK: executable.400.exe
0x000000004a3c00e000 csrss.exe            OK: executable.408.exe
0x00000000ffa10000 winlogon.exe         OK: executable.444.exe
[snip]
```

It failed to extract the `System` process, but that’s normal—this process has no corresponding executable. Notice the name of the output file is based on the PID of the process (`executable.PID.exe`). Here’s another example showing how to extract a process *that’s not in the active process list* based on the physical offset of its `_EPROCESS` (which you can get with `psscan` or `psxview`):

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
     procdump --offset=0x000000003e1e6b30 
     –-dump-dir=OUTDIR/

Volatility Foundation Volatility Framework 2.4
Process(V)         ImageBase          Name         Result
------------------ ------------------ ------------ ------
0xfffffa8002be6b30 0x0000000000400000 warrant.exe  OK: executable.3036.exe
```

The next command extracts any DLL from PID 1408 that has a name or path matching the case-insensitive “crypt” string. The output files are named according to the PID and physical offset of the host process and the base virtual address of the DLL (`module.PID.OFFSET.ADDRESS.dll`):

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
     dlldump -p 1408 --regex=crypt 
     --ignore-case 
     --dump-dir=OUTDIR/ 

Volatility Foundation Volatility Framework 2.4
Name          Module Base   Module Name   Result
------------- ------------- ------------- ------
explorer.exe  0x7fefd130000 CRYPTSP.dll   module.1408.3e290b30.7fefd130000.dll
explorer.exe  0x7fefc5c0000 CRYPTUI.dll   module.1408.3e290b30.7fefc5c0000.dll
explorer.exe  0x7fefd7c00e000 CRYPTBASE.dll module.1408.3e290b30.7fefd7c00e000.dll
explorer.exe  0x7fefdb30000 CRYPT32.dll   module.1408.3e290b30.7fefdb30000.dll
```

The regular expression option works only with DLLs in the PEB lists. You cannot dump hidden or injected DLLs by name because often you don’t have a name—only a base address where the DOS header exists. In those cases, you can specify the `--base` like this:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
     dlldump -p 1148 --base=0x000007fef7310000 
     --dump-dir=OUTDIR/ 
     --memory

Volatility Foundation Volatility Framework 2.4
Name          Module Base   Module Name Result
------------- ------------- ----------- ------
spoolsv.exe   0x7fef7310000 UNKNOWN     module.1148.3e79bb30.7fef7310000.dll
```

---

**NOTE**

This book doesn’t cover live analysis tools in detail, but it’s worth noting that the following tools can help you dump processes from running systems:

- Sysinternals ProcDump and Process Explorer: [http://technet.microsoft.com/en-us/sysinternals/dd996900.aspx](http://technet.microsoft.com/en-us/sysinternals/dd996900.aspx)
- Process Hacker: [http://processhacker.sourceforge.net](http://processhacker.sourceforge.net)

---

### Caveats and Workarounds

One thing to note about these plugins is that they’re susceptible to attacks that manipulate PE header values. For example, if the `MZ` or `PE` signature isn’t found, they cannot properly locate the sections. Furthermore, they also rely on the *advertised* section virtual addresses and sizes, which malicious code can easily overwrite. If a section claims to be much larger or smaller than it actually is, the output file will have either extraneous data or missing data. Another reason why these plugins can fail is simply due to chance. If the page(s) containing the PE header or section information are swapped to disk, the header validation routines fail.

If you encounter issues dumping PE files from process memory, whether it’s the process executable or a DLL, you can always fall back to just dumping the containing VAD region. If you remember from Chapter 7, the `vaddump` plugin produces a padded file to maintain spatial integrity, and it does not know (or care) about the PE file format. This enables you to still acquire the data even if it has been intentionally corrupted. However, it might require manual fix-ups before it loads in external tools such as IDA Pro or PE viewers. Another alternative is to use the `dumpfiles` plugin (see Chapter 16) that extracts cached copies of the PE files from disk. In this case, you won’t get slack space, modified global variables, or decompressed versions of packed files.

## Packing and Compression

The layers of obfuscation introduced by packed or compressed binaries are often removed when they load into memory. In almost all cases, before executing the main payload, a self-modifying program decompresses in place, or moves to another address and then decompresses. [Figure 8-5](#figure8-5) shows the life cycle of a binary that gets packed and then loaded in memory. Before packing, the PE file’s entry point is within its `.text` section (likely the `main` or `DllMain` function), and all strings and code are in plain text. You can perform static analysis of the binary at this stage. However, after it’s packed, the strings and code are compressed and optionally encrypted or encoded. Also, a new section has been added to the binary that contains the unpacking code, and the entry point now leads to this new section. Statically analyzing the file at this point will most certainly fail.

![c08f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f005.png)

*[**Figure 8-5:**](#figureanchor8-5) The life cycle of a program being packed and loaded into memory*

When the binary is loaded into memory, the entry point function (unpacking code) decompresses the original strings and data so the program can access them. Next the real `main` or `DllMain` routine is called to invoke the program’s normal behavior. Anytime between now and when the DLL unloads or the process terminates, you should be able to dump the decompressed code from memory. Finding it is rarely an issue because you have `dlllist`, `ldrmodules`, and `malfind` (described in the next section) to provide you with base addresses. Paired with `procdump`, `dlldump`, and `moddump`, you have a pretty flexible toolset for acquiring the data you need for static analysis.

---

**NOTE**

Your ability to dump decompressed code depends on the design of the packer. A majority of packers used in the wild operate in a manner consistent with our description, but not all of them. For example, virtual machine (VM) packers such as VMProtect ([http://vmpsoft.com/products/vmprotect](http://vmpsoft.com/products/vmprotect)) and Themida ([http://www.oreans.com/themida.php](http://www.oreans.com/themida.php)) are especially problematic because they never fully unpack in memory. In fact, the code is transformed so that it’s often not technically possible to derive the original code. In these cases, we tend to focus on artifacts left in memory when the programs execute—for example, the network connections, open file handles, services they create, and so on.

---

### Unpacking Malicious Code

This section demonstrates a practical example of using memory analysis to unpack a malware sample. [Figure 8-6](#figure8-6) shows the functions that the packed sample imports from `kernel32.dll` (shown via CFF Explorer from [http://www.ntcore.com/exsuite.php](http://www.ntcore.com/exsuite.php)). You can typically look at a program’s imports and get a general idea of its functionality, but in this case you don’t see much—which indicates the packer obfuscates the IAT as well as the program’s instructions. However, because it includes `GetProcAddress` and `LoadLibraryW`, it can load and access any APIs it needs at run time.

![c08f006.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f006.png)

*[**Figure 8-6:**](#figureanchor8-6) The packed malware’s Import Address Table contains only a few functions.*

If you extract strings from the binary, you’ll see that there aren’t many—besides the DOS message and some partly familiar items at the top. For example, `zirtualAlloc` and `zegOpenKeyExW` closely resemble `VirtualAlloc` and `RegOpenKeyExW` (names of Windows API functions). These are suspicious indeed and they probably indicate an attempt to bypass string-based signatures:

```
$ strings -a -n 8 734aa.ex_ > strings.txt
$ strings -a -n 8 –el 734aa.ex_ >> strings.txt
$ cat strings.txt
!This program cannot be run in DOS mode.
PUSHBUTTON
zirtualAlloc
zegOpenKeyExW

[snip]

X660~6B0&6v0
1C55145F1
KhdoUuDiPcctMtyAb
%ctM5bul
j32Dbllb
wsvcPr.dFj

[snip]

9#949A9N9v9{9
8$8*80868<8B8H8N8T8Z8`8f8l8r8x8~8
120609072050Z
391231235959Z0
```

At this point, it’s pretty obvious that the sample is packed. You can execute it in a controlled environment (such as a VM with networking disabled) to allow the program to unpack in memory. While it’s running, capture its state by taking a snapshot of your VM or dumping memory. Then use `procdump` to extract the decompressed code, as shown here:

```
$ python vol.py -f infected.vmem procdump -p 3060 --dump-dir=OUTDIR/
Volatility Foundation Volatility Framework 2.4
Process(V) ImageBase  Name       Result
---------- ---------- ---------- ------
0x81690c10 0x00400000 734aa.ex_    OK: executable.3060.exe
```

Now that you have a sample extracted from memory, run strings on it again and see the difference:

```
$ strings -a -n 8 executable.3060.exe > strings.txt
$ strings -a -n 8 –el executable.3060.exe >> strings.txt
$ cat strings.txt
!This program cannot be run in DOS mode.

[snip]

Software\Microsoft\WSH\
Mozilla\Firefox\Profiles
cookies.*
Macromedia
firefox.exe
iexplore.exe
explorer.exe
Content-Length

[snip]

http://REDACTED.65.40:8080/zb/v_01_a/in/
http://REDACTED.189.124:8080/zb/v_01_a/in/
http://REDACTED.154.199:8080/zb/v_01_a/in/
http://REDACTED.150.163:8080/zb/v_01_a/in/
Software\Microsoft\Windows\CurrentVersion\Run
[snip]
```

The output tells you a lot about the malware’s functionality. Although they are just strings, not direct code correlations, sometimes that’s all you need to generate an initial list of indicators such as contacted IPs and modified registry keys.

### Common Unpacking Issues

You can save an enormous amount of time by just letting the malware natively unpack in memory as opposed to using a debugger to reverse-engineer samples, but there are some disadvantages as well. For example, as previously mentioned, some samples never fully unpack. You also might run into samples that don’t stay active long enough for you to capture memory. In those cases, we recommend using a debugger to run the malware and set a breakpoint on `ExitProcess`: It gets “frozen” while you access what you need. Of course, you can also find debugger-aware samples, so make sure to read Recipe 9-11 (“Preventing Processes from Terminating”) in *Malware Analyst’s Cookbook.* It shows you how to suspend a process when it tries to exit by placing hooks in the kernel.

### Unpacking 64-bit DLLs

Unpacking DLLs on 64-bit Windows is technically no different from executables on 32-bit platforms. As we were preparing to write this section of the chapter, we received a Rovnix ([http://www.xylibox.com/2013/10/reversible-rovnix-passwords.html](http://www.xylibox.com/2013/10/reversible-rovnix-passwords.html)) sample that was implemented as a DLL and targeted 64-bit systems. As shown in [Figure 8-7](#figure8-7), the sample was packed. If you’re not familiar with IDA Pro, the horizontal bar below the main menu (known as the color bar) displays the segments in the file that contain instructions versus undefined data. A majority of this file was undefined. Furthermore, the entry point function calls `VirtualAlloc`—probably to allocate a memory region to use as the “scratch pad” where decompression can occur. You can also see 62 exported functions with names that resemble database APIs. Viewing their code (not shown here), you see that they consist of only no-operations (NOPs).

![c08f007.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f007.png)

*[**Figure 8-7:**](#figureanchor8-7) The DLL loaded in IDA Pro shows various signs of being compressed or encrypted.*

Chapter 13 (“Working with DLLs”) in *Malware Analyst’s Cookbook* contains some useful hints that you can leverage here. The goal is to get the DLL loaded so that it can unpack and then keep it there long enough for you to acquire memory. We use `rundll32.exe` (which is actually a 64-bit executable on 64-bit platforms, despite its name) as the host process—it takes care of mapping the malicious DLL in memory and calling its entry point function (`DllMain`). Here is the command we use:

```
C:\Users\Elliot\Desktop\> rundll32 dd4382d225a[snip].dll,FakeExport
```

---

**NOTE**

The name of the exported function to call is `FakeExport`, which is completely inconsequential. By the time `FakeExport` is referenced by `rundll32.exe`, the malicious DLL is already loaded. It could easily be one of the exports you see in [Figure 8-7](#figure8-7), or you could make up another term to use. Strangely, you can’t just leave off the export name or else you get a syntax error from `rundll32.exe`.

---

With the DLL loaded, capture memory and then determine its base address with `dlllist`, as shown here:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 -p 1524 dlllist
Volatility Foundation Volatility Framework 2.4 
************************************************************************
rundll32.exe pid:   1524
Command line : rundll32  dd4382d225a15dc09f92616131eff983.dll,FakeEntry
Service Pack 1

Base                Size    LoadCount Path
------------------ -------  ------------------ ----
0x00000000ffbc00f000   0xf000  0xffff C:\Windows\system32\rundll32.exe
0x0000000077040000 0x1a9000  0xffff C:\Windows\SYSTEM32\ntdll.dll
0x0000000076e20000 0x11f000  0xffff C:\Windows\system32\kernel32.dll
[snip]
0x0000000180000000  0x15000     0x1 C:\Users\Elliot\Desktop\dd438[snip].dll
0x000007fefe090000  0xdb000     0x2 C:\Windows\system32\ADVAPI32.dll
0x000007fefe040000  0x1f000     0x8 C:\Windows\SYSTEM32\sechost.dll
0x000007fefde50000 0x12d000     0x6 C:\Windows\system32\RPCRT4.dll
[snip]
```

The output tells you that the DLL is loaded at base address `0x180000000` in the process. Now you can go ahead and dump the DLL like this:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 -p 1524 dlldump 
     --base 0x0000000180000000 
     –-dump-dir=OUTDIR/ 

Volatility Foundation Volatility Framework 2.4
Name          Module Base        Module Name        Result
------------- ------------------ ------------------ ------
rundll32.exe  0x0000000180000000 dd438[snip].dll    OK: 
module.1524.3f769610.180000000.dll
```

When the dumped file is loaded in IDA Pro this time, you see significant differences. Notice how the unpacked copy in [Figure 8-8](#figure8-8) compares with the packed copy previously shown in [Figure 8-7](#figure8-7).

The color bar has changed to indicate that a majority of the file contains code instead of undefined data. The strange database–related export functions are gone, the import table is intact, and (most importantly) the instructions in the disassembly pane are no longer compressed.

![c08f008.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f008.png)

*[**Figure 8-8:**](#figureanchor8-8) The unpacked DLL in IDA Pro is now in good enough shape to allow static analysis.*

## Code Injection

Malware leverages code injection to perform actions from within the context of another process. By doing so, the malware can force a legitimate process to perform actions on its behalf, such as downloading additional trojans or stealing information from the system. Attackers can inject code into a process in many ways, such as writing to the remote process’ memory directly or adding a registry key that makes new processes load a DLL of the attacker’s choice. This section discusses how you can determine whether any processes on the system are victims of code injection, and if so, how you can extract the memory segments that contain malicious code.

The classes of code injection that we cover are these:

- **Remote DLL injection:** A malicious process forces the target process to load a specified DLL from disk by calling `LoadLibrary` or the native `LdrLoadDll`. By definition, the DLL *must* exist on disk prior to being injected.
- **Remote code injection:** A malicious process writes code into the memory space of a target process and forces it to execute. The code can be a block of shellcode (i.e., not a PE file) or it can be a PE file whose import table is preemptively configured for the target process.
- **Reflective DLL injection:** A malicious process writes a DLL (as a sequence of bytes) into the memory space of a target process. The DLL handles its own initialization without the help of the Windows loader. The DLL does *not* need to exist on disk prior to being injected.
- **Hollow process injection:** A malicious process starts a new instance of a legitimate process (such as `lsass.exe`) in suspended mode. Before resuming it, the executable section(s) are freed and reallocated with malicious code.

As you will learn, the technique you use to detect code injection depends on how the code was injected, which is the reason we distinguish between the different methods. Although Volatility provides the capability to detect all types, you still need to put in some analysis effort to learn when to use the various plugins and how to properly interpret their output.

In the following descriptions, Process A is the malicious process and Process B is the target.

### Remote DLL Injection

This technique is typically accomplished in the following steps:

1. Process A enables debug privilege (`SE_DEBUG_PRIVILEGE`) that gives it the right to read and write other process’ memory as if it were a debugger.
2. Process A opens a handle to Process B by calling `OpenProcess`. It must request at least `PROCESS_CREATE_THREAD`, `PROCESS_VM_OPERATION`, and `PROCESS_VM_WRITE`.
3. Process A allocates memory in Process B using `VirtualAllocEx`. The protection is typically `PAGE_READWRITE`.
4. Process A transfers a string to Process B’s memory by calling `WriteProcessMemory`. The string identifies the full path on disk to the malicious DLL and it is written at the address allocated in the previous step.
5. Process A calls `CreateRemoteThread` to start a new thread in Process B that executes the `LoadLibrary` function. The thread’s parameter is set to the full path to the malicious DLL, which already exists in Process B’s memory.
6. At this point, the injection is complete and Process B has loaded the DLL. Process A calls `VirtualFree` to free the memory containing the DLL’s path.
7. Process A calls `CloseHandle` on Process B’s process to clean up.

### DLL Injection Detection

Considering the fact that `LoadLibrary` was used to load the DLL, there is no good way to conclusively distinguish between the malicious DLL and other explicitly loaded DLLs in Process B. The VAD and PEB lists look nearly identical from a metadata perspective for all modules loaded with the same API. In other words, the injected DLL isn’t necessarily hidden at this point; it is perfectly visible with `dlllist` or tools such as Process Explorer running on the live system. However, unless you know the specific name of the DLL, it can easily blend in with the legitimate modules.

Two factors can make detection possible. First, if the injected DLL *does* attempt to hide from tools on the live system after it gets loaded (by unlinking its `_LDR_DATA_TABLE_ENTRY` from one or more of the ordered lists), you can use `ldrmodules` to detect it. The second is if the injected DLL is packed, and the unpacking procedure copies the decompressed code to a new memory region. In this case, you’ll detect it with `malfind` (described next). If neither of these things happens, you have to fall back to typical analysis involving context, path names, Yara scans, DLL load timestamps (see Chapter 18), and so on.

### Remote Code Injection

This technique starts out with the same two steps as remote DLL injection. Process A enables debug privilege and then opens a handle to Process B. It finishes up like this:

1. Process A allocates memory in Process B with the `PAGE_EXECUTE_READWRITE` protection. This protection level is necessary to allow Process A to write to the memory and Process B to read and execute it.
2. Process A transfers a block of code to Process B using `WriteProcessMemory`.
3. Process A calls `CreateRemoteThread` and points the thread’s starting address to a function within the transferred block of code.

### Code Injection Detection

The `malfind` plugin is designed to hunt down remote code injections that occur as previously described. We hinted about this many times in Chapter 7, which discussed the VAD characteristics and flags. The concept is that there will be a readable, writeable, and executable private memory region (that is, no file mapping) with all pages committed (we use a few variations of these criteria for detection). The region will contain a PE header and/or valid CPU instructions. Here’s an example showing a block of code that Stuxnet injected into `services.exe`:

```
$ python vol.py –f stuxnet.mem --profile=WinXPSP3x86 malfind
Volatility Foundation Volatility Framework 2.4

[snip]

Process: services.exe Pid: 668 Address: 0x13c00f000
Vad Tag: Vad  Protection: PAGE_EXECUTE_READWRITE
Flags: Protection: 6

0x013c00f000  4d 5a 90 00 03 00 00 00 04 00 00 00 ff ff 00 00   MZ..............
0x013c00f010  b8 00 00 00 00 00 00 00 40 00 00 00 00 00 00 00   ........@.......
0x013c00f020  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x013c00f030  00 00 00 00 00 00 00 00 00 00 00 00 08 01 00 00   ................

0x13c00f000 4d               DEC EBP
0x13c00f001 5a               POP EDX
0x13c00f002 90               NOP
0x13c00f003 0003             ADD [EBX], AL
0x13c00f005 0000             ADD [EAX], AL
0x13c00f007 000400           ADD [EAX+EAX], AL
```

The output shows a preview of the data as both a hex dump and disassembly, starting at the base address of the injected region (`0x013c00f000`). In some cases, you’ll leverage the hex dump to determine whether the region is malicious (for example, because you see an `MZ` signature); in other cases, you’ll need to rely on the disassembly. Here’s an example that shows the disassembly coming in handy. In the following detection of Carberp, no `MZ` signature exists because it is just a block of shellcode:

```
$ python vol.py –f carberp.mem --profile=WinXPSP3x86 malfind
Volatility Foundation Volatility Framework 2.4 
[snip]

Process: svchost.exe Pid: 992 Address: 0x9d0000
Vad Tag: VadS Protection: PAGE_EXECUTE_READWRITE
Flags: CommitCharge: 1, MemCommit: 1, PrivateMemory: 1, Protection: 6

0x009d0000  b8 35 00 00 00 e9 8b d1 f3 7b 68 6c 02 00 00 e9   .5.......{hl....
0x009d0010  94 63 f4 7b 8b ff 55 8b ec e9 6c 11 e4 7b 8b ff   .c.{..U...l..{..
0x009d0020  55 8b ec e9 99 2e 84 76 8b ff 55 8b ec e9 74 60   U......v..U...t`
0x009d0030  7f 76 8b ff 55 8b ec e9 8a e9 7f 76 8b ff 55 8b   .v..U......v..U.

0x9d0000 b835000000       MOV EAX, 0x35
0x9d0005 e98bd1f37b       JMP 0x7c90d195
0x9d000a 686c020000       PUSH DWORD 0x26c
0x9d000f e99463f47b       JMP 0x7c9163a8
0x9d0014 8bff             MOV EDI, EDI
0x9d0016 55               PUSH EBP
```

This region at `0x9d0000` is worth further investigation because the disassembly contains CPU instructions that make sense. For example, the `JMP` destinations are valid, and the combination of `MOV EDI, EDI` followed by `PUSH EBP` indicates the start of a function prologue. When you review the output of `malfind`, keep in mind that programs may allocate executable private memory for legitimate reasons. For example, the following region in `csrss.exe` was *not* injected by malware; it was picked up by the plugin due to its similarity to injection regions:

```
Process: csrss.exe Pid: 660 Address: 0x7f6c00f000
Vad Tag: Vad  Protection: PAGE_EXECUTE_READWRITE
Flags: Protection: 6

0x7f6c00f000  c8 00 00 00 71 01 00 00 ff ee ff ee 08 70 00 00   ....q........p..
0x7f6c00f010  08 00 00 00 00 fe 00 00 00 00 10 00 00 20 00 00   ................
0x7f6c00f020  00 02 00 00 00 20 00 00 8d 01 00 00 ff ef fd 7f   ................
0x7f6c00f030  03 00 08 06 00 00 00 00 00 00 00 00 00 00 00 00   ................

0x7f6c00f000 c8000000         ENTER 0x0, 0x0
0x7f6c00f004 7101             JNO 0x7f6c00f007
0x7f6c00f006 0000             ADD [EAX], AL
0x7f6c00f008 ff               DB 0xff
0x7f6c00f009 ee               OUT DX, AL
0x7f6f000a ff               DB 0xff
0x7f6f000b ee               OUT DX, AL
```

The disassembly does not make sense in this case. For example, there’s an `ENTER` instruction, but no `LEAVE`. There’s conditional jump (`JNO`), but no condition. Furthermore, the destination of the jump leads to `0x7f6c00f007`, an address that does not contain an instruction according to the current alignment. This memory region does not seem malicious at first glance. Consider the next example, involving Coreflood:

```
$ python vol.py –f coreflood.mem --profile=WinXPSP3x86 malfind
Volatility Foundation Volatility Framework 2.4

[snip]

Process: IEXPLORE.EXE Pid: 248 Address: 0x7ff80000
Vad Tag: VadS Protection: PAGE_EXECUTE_READWRITE
Flags: CommitCharge: 45, PrivateMemory: 1, Protection: 6

0x7ff80000  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x7ff80010  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x7ff80020  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x7ff80030  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................

0x7ff80000 0000             ADD [EAX], AL
0x7ff80002 0000             ADD [EAX], AL
0x7ff80004 0000             ADD [EAX], AL
0x7ff80006 0000             ADD [EAX], AL
```

Most analysts assume that this range at `0x7ff80000` is a false positive. The hex dump and disassembly both consist of only zeros. However, remember that this is only a preview of the data. The CPU doesn’t necessarily start executing code at offset 0 in the injected region; it can easily point somewhere within the range. In this case, Coreflood’s anti-dumping feature wiped out its PE header (which occupied the first page) by overwriting it with zeros. If you use `volshell` to disassemble code in the second page (`0x7ff81000`), however, you see the malware’s main function:

```
$ python vol.py -f coreflood.mem --profile=WinXPSP3x86 volshell -p 248
Volatility Foundation Volatility Framework 2.4
Current context: process IEXPLORE.EXE, pid=248, ppid=1624 DTB=0x80002a0
To get help, type 'hh()'
>>> dis(0x7ff81000)
0x7ff81000 81ec20010000                     SUB ESP, 0x120
0x7ff81006 53                               PUSH EBX
0x7ff81007 8b9c2430010000                   MOV EBX, [ESP+0x130]
0x7fc81f000e 8bc3                             MOV EAX, EBX
0x7ff81010 2404                             AND AL, 0x4
0x7ff81012 55                               PUSH EBP
0x7ff81013 f6d8                             NEG AL
0x7ff81015 56                               PUSH ESI
0x7ff81016 57                               PUSH EDI
0x7ff81017 8bbc2434010000                   MOV EDI, [ESP+0x134]
[snip]
```

Malware can use several tricks to hide in process memory. The task of verifying code in these regions often requires a familiarity with assembly because injected code can look very similar to legitimate code. After `malfind` helps you narrow down the possibilities to a manageable level, you still have to apply some knowledge and context to the investigation. You can also supply an output directory (`--dump-dir`) when calling `malfind`, and it extracts the suspect regions automatically. Then you can run strings or signature scanners across the dumped files.

---

**WARNING**

Coreflood prevented many tools from carving its binary from memory by wiping out its PE header. This anti-forensics approach is an annoyance, but it shouldn’t stop you. Just use `vaddump` to extract the memory region and use a hex editor or PE editor to build your own PE header template. For more information, see *Recovering Coreflood Binaries with Volatility*: [http://mnin.blogspot.com/2008/11/recovering-coreflood-binaries-with.html](http://mnin.blogspot.com/2008/11/recovering-coreflood-binaries-with.html).

---

### Reflective DLL Injection

This method is a hybrid of the two approaches discussed previously. The content transferred from Process A to Process B is a DLL (as opposed to a block of shellcode), but after it exists in Process B, it initializes itself instead of calling `LoadLibrary`. This technique has several anti-forensic advantages:

- `LoadLibrary` only loads libraries from disk. Because this method doesn’t rely on the API, the injected DLL never needs to be written to more permanent storage. It can be loaded into memory straight from the network (for example, when exploiting a remote buffer overflow).
- Also as a result of avoiding `LoadLibrary`, the `_LDR_DATA_TABLE_ENTRY` metadata structures are not created. Thus the three lists in the PEB do not have a record of this DLL loading.

---

**NOTE**

For more information on this technique, see:

- *Remote Library Injection* by skape ([http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf](http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf)).
- *Reflective DLL Injection* by Steven Fewer: ([http://www.harmonysecurity.com/files/HS-P005_ReflectiveDllInjection.pdf](http://www.harmonysecurity.com/files/HS-P005_ReflectiveDllInjection.pdf)).
- You can also check out source code and compile your own test binaries from the following repository: [https://github.com/stephenfewer/ReflectiveDLLInjection](https://github.com/stephenfewer/ReflectiveDLLInjection).

---

### Reflective DLL Injection Detection

You can detect this method with `malfind`, as discussed previously. Here’s a snippet of code from the `ReflectiveDLLInjection` project’s `LoadLibraryR.c` file:

```
// alloc memory (RWX) in the host process for the image...
lpRemoteLibraryBuffer = VirtualAllocEx( hProcess, 
                                        NULL, 
                                        dwLength, 
                                        MEM_RESERVE|MEM_COMMIT, 
                                        PAGE_EXECUTE_READWRITE ); 
if( !lpRemoteLibraryBuffer )
    break;
```

Due to the options chosen during allocation, the VAD in the host process that contains the DLL fits the criteria for `malfind`.

---

**NOTE**

Metasploit’s VNC and Meterpreter payloads are both based on the reflective DLL injection method. For more information, see [http://www.offensive-security.com/metasploit-unleashed/Payload_Types](http://www.offensive-security.com/metasploit-unleashed/Payload_Types). You should be able to detect these types of attacks in memory using the plugins discussed in this chapter.

---

### Hollow Process Injection

With the previously discussed methods of injection, the target process remains running and just executes additional (malicious) code on behalf of the malware. On the other hand, with process hollowing, the malware starts a new instance of a legitimate process, such as `lsass.exe`. Before the process’ first thread begins, the malware frees the memory containing the `lsass.exe` code (it hollows it out) and replaces it with the body of the malware. In this sense, it executes only malicious code for the remainder of the process’ lifetime. However, the PEB and various other data structures identify the path to the legitimate `lsass.exe` binary. [Figure 8-9](#figure8-9) shows a before-and-after memory layout for the described behavior.

![c08f009.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f009.png)

*[**Figure 8-9:**](#figureanchor8-9) When a process is hollowed, its executable section is replaced with malicious code.*

#### How to Hollow a Process

The following steps describe how to conduct such an attack. Recipe 15-8 in *Malware Analyst’s Cookbook* also includes the relevant C source code for each step.

1. Start a new instance of a legitimate process (for example, `C:\windows\system32\lsass.exe`), but with its first thread suspended. At this point, the `ImagePathName` in the PEB of the new process identifies the full path to the legitimate `lsass.exe`.
2. Acquire the contents for the malicious replacement code. This content can come from a file on disk, an existing buffer in memory, or over the network.
3. Determine the base address (`ImageBase`) of the `lsass.exe` process, and then free or unmap the containing memory section. At this point, the process is just an empty container (the DLLs, heaps, stacks, and open handles are still intact, but no process executable exists).
4. Allocate a new memory segment in `lsass.exe` and make sure that the memory can be read, written, and executed. You can reuse the same `ImageBase` or a different one
5. Copy the PE header for the malicious process into the newly allocated memory in `lsass.exe`.
6. Copy each PE section for the malicious process into the proper virtual address in `lsass.exe`.
7. Set the start address for the first thread (the one that has been in a suspended state) to point at the malicious process’ `AddressOfEntryPoint` value.
8. Resume the thread. At this point, the malicious process begins executing within the container created for `lsass.exe`. The `ImagePathName` in the PEB still points to `C:\windows\system32\lsass.exe`.

#### Detection

Stuxnet creates two new instances of `lsass.exe` and replaces their code, just as the previous steps described. When you list processes, you see the following:

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 pslist | grep lsass
Volatility Foundation Volatility Framework 2.4
Offset(V)  Name        PID   PPID   Thds     Hnds   Start                       
---------- ----------- ------ ------ ------ ------  ----------------------------
0x81e70020 lsass.exe      680    624     19    342  2010-10-29 17:08:54 UTC+0000
0x81c498c8 lsass.exe      868    668      2     23  2011-06-03 04:26:55 UTC+0000
0x81c47c00 lsass.exe     1928    668      4     65  2011-06-03 04:26:55 UTC+0000
```

There are three processes (PID 680, 868, and 1928), but only one is the “real” `lsass.exe`. Intuition may tell you that the one that started first based on creation time (PID 680) is the legitimate one, but we’ll show you how to confirm. First, you can view the full path to the executable and/or its command line. The following command shows this information as well as the `ImageBase` value for the processes:

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 dlllist 
      -p 680,868,1928 | grep lsass
Volatility Foundation Volatility Framework 2.4
lsass.exe pid:    680
Command line : C:\WINDOWS\system32\lsass.exe
0x01000000     0x6000     0xffff C:\WINDOWS\system32\lsass.exe

lsass.exe pid:    868
Command line : "C:\WINDOWS\\system32\\lsass.exe"
0x01000000     0x6000     0xffff C:\WINDOWS\system32\lsass.exe

lsass.exe pid:   1928
Command line : "C:\WINDOWS\\system32\\lsass.exe"
0x01000000     0x6000     0xffff C:\WINDOWS\system32\lsass.exe
```

The advertised paths are all the same (despite two having an extra set of quotes around the path) because the data in the PEB, including the `ImageBase`, is initialized at process creation, and all processes started out the same. However, as a result of being hollowed, the VAD characteristics for the region that contains the `ImageBase` are drastically different. Only the legitimate one (PID 680) still has a copy of the `lsass.exe` file mapped into the region:

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 vadinfo 
      -p 1928,868,680 --addr=0x01000000
Volatility Foundation Volatility Framework 2.4
************************************************************************
Pid:    680
VAD node @ 0x81db03c0 Start 0x01000000 End 0x01005fff Tag Vad 
Flags: CommitCharge: 1, ImageMap: 1, Protection: 7
Protection: PAGE_EXECUTE_WRITECOPY
ControlArea @823c40e008 Segment e1735398
NumberOfSectionReferences:          3 NumberOfPfnReferences:           4
NumberOfMappedViews:                1 NumberOfUserReferences:          4
Control Flags: Accessed: 1, File: 1, HadUserReference: 1, Image: 1
FileObject @82230120, Name: \WINDOWS\system32\lsass.exe
First prototype PTE: e17353d8 Last contiguous PTE: fffffffc
Flags2: Inherit: 1

************************************************************************
Pid:    868
VAD node @ 0x81f1ef08 Start 0x01000000 End 0x01005fff Tag Vad 
Flags: CommitCharge: 2, Protection: 6
Protection: PAGE_EXECUTE_READWRITE
ControlArea @81fbeee0 Segment e24b4c10
NumberOfSectionReferences:          1 NumberOfPfnReferences:           0
NumberOfMappedViews:                1 NumberOfUserReferences:          2
Control Flags: Commit: 1, HadUserReference: 1
First prototype PTE: e24b4c50 Last contiguous PTE: e24b4c78
Flags2: Inherit: 1

************************************************************************
Pid:   1928
VAD node @ 0x82086d40 Start 0x01000000 End 0x01005fff Tag Vad 
Flags: CommitCharge: 2, Protection: 6
Protection: PAGE_EXECUTE_READWRITE
ControlArea @81ff33e0 Segment e2343888
NumberOfSectionReferences:          1 NumberOfPfnReferences:           0
NumberOfMappedViews:                1 NumberOfUserReferences:          2
Control Flags: Commit: 1, HadUserReference: 1
First prototype PTE: e23438c8 Last contiguous PTE: e23438f0
Flags2: Inherit: 1
```

As an alternative to the multiple steps you just saw, you can also skip straight to `ldrmodules`. Remember that the process executable is added to the load order and memory order module lists in the PEB. Thus, when `ldrmodules` cross-references the information with the memory-mapped files in the VAD, you see a discrepancy. Here’s an example:

```
$ python vol.py -f stuxnet.vmem ldrmodules --profile=WinXPSP3x86 -p 1928
Volatility Foundation Volatility Framework 2.4
Pid      Process   Base       InLoad InInit InMem MappedPath
-------- --------- ---------- ------ ------ ----- ----------
[snip]
1928 lsass.exe     0x7c900000 True   True   True  \WINDOWS\system32\ntdll.dll
1928 lsass.exe     0x77f60000 True   True   True  \WINDOWS\system32\shlwapi.dll
1928 lsass.exe     0x771b0000 True   True   True  \WINDOWS\system32\wininet.dll
1928 lsass.exe     0x77c00000 True   True   True  \WINDOWS\system32\version.dll
1928 lsass.exe     0x01000000 True   False  True  <no name>
[snip]
```

Because `lsass.exe` was unmapped, a name is no longer associated with the region at `0x01000000`. But calling `NtUnmapViewOfSection` (step 3) doesn’t cause the PEB to lose its metadata, so those structures still have a record of the original mapping in the load order and memory order lists.

---

**NOTE**

For more information on the hollow process technique, see:

- *Analyzing Malware Hollow Process* by Eric Monti: [http://blog.spiderlabs.com/2011/05/analyzing-malware-hollow-processes.html](http://blog.spiderlabs.com/2011/05/analyzing-malware-hollow-processes.html)
- *Debugging Hollow Processes* by Alexander Hanel: [http://hooked-on-mnemonics.blogspot.com/2013/01/debugging-hollow-processes.html](http://hooked-on-mnemonics.blogspot.com/2013/01/debugging-hollow-processes.html)

---

### Postprocessing Dumped Code

After you identify injected code regions, you can extract them to disk for static analysis. In *most* cases, you just need to fix the `ImageBase` in the dumped PE header to match its location in memory and then you can load the file in IDA Pro. If imported functions aren’t visible at that time, use the `impscan` plugin to generate context information from the memory dump and import the data as labels into IDA. The same goes for any process, DLL, or kernel driver that you dump from memory, not just injected regions. Recipe 16-8 (“Scanning for Imported Functions with Impscan”) in *Malware Analyst’s Cookbook* describes this capability in further detail.

Although a majority of code you dump from memory is self-contained (for example, a single region that contains the executable code and read/write variables), that’s not always the case, especially with Poison Ivy when its “melt” functionality is enabled. This feature causes the RAT to dissolve into process memory space, scattering small pieces of its code all over. [Figure 8-10](#figure8-10) shows the effect this has on your ability to reconstruct the original binary for static analysis.

![c08f010.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f010.png)

*[**Figure 8-10:**](#figureanchor8-10) Poison Ivy’s “melt” feature spreads injected code fragments all over process memory.*

Poison Ivy is spread across 20+ different VAD regions. In the article *Reverse Engineering Poison Ivy’s Injected Code Fragments* ([http://volatility-labs.blogspot.com/2012/10/reverse-engineering-poison-ivys.html](http://volatility-labs.blogspot.com/2012/10/reverse-engineering-poison-ivys.html)), we demonstrate how to build a custom plugin for Volatility that reconstructs, as much as possible, the original binary from the fragments and puts them back into context.

---

**WARNING**

Poison Ivy’s fragmented code injection is a powerful anti-forensic technique. Designing the `pivydasm` plugin was not easy and it took a considerable amount of time. To our benefit, although the code was spread about many VAD regions, we could still follow the code by disassembling within `volshell`. Resolving strings and API calls was also possible, because we had access to the entire process’ memory space.

---

## Summary

The skills described in this chapter will help analysts detect and analyze a majority of malicious code in the wild. In particular, you learned to identify injected code, locate suspicious DLLs, dump unpacked binaries, and mine the process environment block for artifacts. As far back as we can remember, malware and rootkits have used techniques to hide from administrative and security tools on running systems; and now you understand how they work. Furthermore, you’ve seen examples of memory forensics applied to cases involving large malware campaigns (such as Zeus and Coreflood) as well as highly targeted attack tools like Stuxnet.
