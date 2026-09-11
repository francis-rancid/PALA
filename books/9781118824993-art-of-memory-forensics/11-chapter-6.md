# Chapter 6  
 Processes, Handles, and Tokens

This chapter combines three of the most common initial steps in an investigation: determining what applications are running, what they’re doing (in terms of access to files, registry keys, and so on), and what security context (or privilege level) they have obtained. In doing so, you’ll also learn how to detect hidden processes, how to link processes to specific user accounts, how to investigate lateral movement across networks, and how to analyze privilege escalation attacks.

Although this chapter covers a wide range of process-related investigation techniques, it’s only the beginning—and it mainly deals with artifacts that exist in kernel memory. The analysis methods involving dynamic link libraries (DLLs), process memory, injected code, and things of that nature are covered in the chapters that follow.

## Processes

The diagram in [Figure 6-1](#figure6-1) shows several of the basic resources that belong to a process. At the center is the `_EPROCESS`, which is the name of the structure that Windows uses to represent a process. Although the structure names certainly differ among Windows, Linux, and Mac, all operating systems share the same concepts that are described in this high-level diagram. For example, they all have one or more threads that execute code, and they all have a table of handles (or file descriptors) to kernel objects such as files, network sockets, and mutexes.

Each process has its own private virtual memory space that’s isolated from other processes. Inside this memory space, you can find the process executable; its list of loaded modules (DLLs or shared libraries); and its stacks, heaps, and allocated memory regions containing everything from user input to application-specific data structures (such as SQL tables, Internet history logs, and configuration files). Windows organizes the memory regions using virtual address descriptors (VADs), which are discussed in Chapter 7.

![c06f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f001.png)

*[**Figure 6-1:**](#figureanchor6-1) A high-level diagram showing basic process resources*

As the figure also shows, each `_EPROCESS` points to a list of security identifiers (SIDs) and privilege data. This is one of the primary ways the kernel enforces security and access control. By combining all these concepts into your investigative procedure, you can gather a significant amount of evidence to help determine what process(es) were involved in malicious activity, what artifacts are related to an incident, and what user accounts might have been compromised.

---

## **Analysis Objectives**

Your objectives are these:

- **Process internals:** Learn how the operating system keeps track of processes and how Windows APIs enumerate them. This helps you understand why live tools are so easily deceived.
- **Identify critical processes:** Explore several critical Windows processes and learn how normal systems operate. You’ll be more prepared to spot anomalies, especially those that involve attempts to blend in with critical processes.
- **Generate visualizations:** Learn how to create visualizations that illustrate parent and child relationships between processes. Doing so helps you determine the chain of events that led to a particular process starting, which is often very important when piecing together an incident.
- **Detect Direct Kernel Object Manipulation (DKOM):** Spot attempts to hide processes by altering one or more process lists in kernel memory. Specifically, you’ll learn seven different ways to locate processes in memory dumps, which gives you a huge advantage against rootkits.

## **Data Structures**

Windows tracks processes by assigning them a unique `_EPROCESS` structure that resides in a non-paged pool of kernel memory. Here is how it appears for a 64-bit Windows 7 system:

```
>>> dt("_EPROCESS")
'_EPROCESS' (1232 bytes)
0x0   : Pcb                         ['_KPROCESS']
0x160 : ProcessLock                 ['_EX_PUSH_LOCK']
0x168 : CreateTime                  ['WinTimeStamp', {'is_utc': True}]
0x170 : ExitTime                    ['WinTimeStamp', {'is_utc': True}]
0x178 : RundownProtect              ['_EX_RUNDOWN_REF']
0x180 : UniqueProcessId             ['unsigned int']
0x188 : ActiveProcessLinks          ['_LIST_ENTRY']
0x198 : ProcessQuotaUsage           ['array', 2, ['unsigned long long']]
0x1a8 : ProcessQuotaPeak            ['array', 2, ['unsigned long long']]
0x1b8 : CommitCharge                ['unsigned long long']
0x1c0 : QuotaBlock                  ['pointer64', ['_EPROCESS_QUOTA_BLOCK']]
0x1c8 : CpuQuotaBlock               ['pointer64', ['_PS_CPU_QUOTA_BLOCK']]
0x1d0 : PeakVirtualSize             ['unsigned long long']
0x1d8 : VirtualSize                 ['unsigned long long']
0x1e0 : SessionProcessLinks         ['_LIST_ENTRY']
0x1f0 : DebugPort                   ['pointer64', ['void']]
[snip]
0x200 : ObjectTable                 ['pointer64', ['_HANDLE_TABLE']]
0x208 : Token                       ['_EX_FAST_REF']
0x210 : WorkingSetPage              ['unsigned long long']
0x218 : AddressCreationLock         ['_EX_PUSH_LOCK']
[snip]
0x290 : InheritedFromUniqueProcessId   ['unsigned int']
[snip]
0x2d0 : PageDirectoryPte            ['_HARDWARE_PTE']
0x2d8 : Session                     ['pointer64', ['void']]
0x2e0 : ImageFileName               ['String', {'length': 16}]
0x2ef : PriorityClass               ['unsigned char']
0x2f0 : JobLinks                    ['_LIST_ENTRY']
0x300 : LockedPagesList             ['pointer64', ['void']]
0x308 : ThreadListHead              ['_LIST_ENTRY']
0x318 : SecurityPort                ['pointer64', ['void']]
0x320 : Wow64Process                ['pointer64', ['void']]
0x328 : ActiveThreads               ['unsigned long']
0x32c : ImagePathHash               ['unsigned long']
0x330 : DefaultHardErrorProcessing  ['unsigned long']
0x334 : LastThreadExitStatus        ['long']
0x338 : Peb                         ['pointer64', ['_PEB']]
[snip]
0x444 : ExitStatus                  ['long']
0x448 : VadRoot                     ['_MM_AVL_TABLE']
0x488 : AlpcContext                 ['_ALPC_PROCESS_CONTEXT']
0x4a8 : TimerResolutionLink         ['_LIST_ENTRY']
0x4b8 : RequestedTimerResolution    ['unsigned long']
0x4bc : ActiveThreadsHighWatermark  ['unsigned long']
0x4c0 : SmallestTimerResolution     ['unsigned long']
```

## **Key Points**

The key points are these:

- `Pcb`: The kernel’s process control block (`_KPROCESS`). This structure is found at the base of `_EPROCESS` and contains several critical fields, including the `DirectoryTableBase` for address translation and the amount of time the process has spent in kernel mode and user mode.
- `CreateTime`: A UTC timestamp indicating when the process first started.
- `ExitTime`: A UTC timestamp indicating the time the process exited. This value is zero for still-running processes.
- `UniqueProcessId`: An integer that uniquely identifies the process (also known as the *PID*).
- `ActiveProcessLinks`: The doubly linked list that chains together active processes on the machine. Most APIs on a running system rely on walking this list.
- `SessionProcessLinks`: Another doubly linked list that chains together processes in the same session.
- `InheritedFromUniqueProcessId`: An integer that specifies the PID of the parent process. After a process is running, this member is not modified, even if its parent terminates.
- `Session`: This member points to the `_MM_SESSION_SPACE` structure (see Chapter 14) that stores information on a user’s logon session and graphical user interface (GUI) objects.
- `ImageFileName`: The filename portion of the process’ executable. This field stores the first 16 ASCII characters, so longer filenames will appear truncated. To get the full path to the executable, or to see the Unicode name, you can access the corresponding VAD node or members in the PEB (see Chapter 7).
- `ThreadListHead`: A doubly linked list that chains together all the process’ threads (each list element is an `_ETHREAD`).
- ActiveThreads: An integer indicating the number of active threads running in the process context. Seeing a process with zero active threads is a good sign that the process has exited.
- `Peb`: A pointer to the Process Environment Block (PEB). Although this member (`_EPROCESS.Peb`) exists in kernel mode, it points to an address in user mode. The PEB contains pointers to the process’ DLL lists, current working directory, command line arguments, environment variables, heaps, and standard handles.
- `VadRoot`: The root node of the VAD tree. It contains detailed information about a process’ allocated memory segments, including the original access permissions (read, write, execute) and whether a file is mapped into the region.

---

### Process Organization

The `_EPROCESS` structure contains a _`LIST_ENTRY` structure called `ActiveProcessLinks`. The _`LIST_ENTRY` structure contains two members: a `Flink` (forward link) that points to the `_LIST_ENTRY` of the *next* `_EPROCESS` structure, and the `Blink` (backward link) that points to the `_LIST_ENTRY` of the *previous* `_EPROCESS` structure. Together, these items create a chain of process objects, also called a doubly linked list (see Chapter 2). [Figure 6-2](#figure6-2) shows a diagram of how the `_LIST_ENTRY` structures link processes together.

![c06f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f002.png)

*[**Figure 6-2:**](#figureanchor6-2) Processes are chained together in a doubly linked list pointed to by PsActiveProcessHead.*

On a running system, tools, such as Process Explorer and Task Manager, rely on walking the doubly linked list of `_EPROCESS` structures. An API commonly used for this purpose is `NtQuerySystemInformation`, but many higher-level APIs provided by the operating system also access the same data.

### Enumerating Processes in Memory

As briefly described in the “Issues with Profile Selection” section of Chapter 3, to list processes, Volatility first locates the kernel debugger data block (`_KDDEBUGGER_DATA64`). From there, it accesses the `PsActiveProcessHead` member, which points to the head of the doubly linked list of `_EPROCESS` structures. We also discussed the pool-scanning approach in Chapter 5.

In this chapter, we present *many* other ways to find processes in a memory dump. It is important to implement alternative methods because the debugger data block, the linked-list pointers, and the pool tags are all nonessential to OS stability—which means they can be manipulated (accidentally or intentionally) to defeat forensic tools without disrupting the system or its processes.

### Critical System Processes

Before you begin analyzing the state of a system based on what applications are running, you should be familiar with the critical system processes; if you know what’s normal, you can detect what’s abnormal more quickly. Throughout the rest of the chapter, we discuss various practical investigative steps to verify the material you see here, so for now we’ll just focus on the theoretical concepts of how things *should* appear on clean systems.

---

**NOTE**

Patrick Olsen’s *Know your Windows Processes or Die Trying* ([http://sysforensics.org/2014/01/know-your-windows-processes.html](http://sysforensics.org/2014/01/know-your-windows-processes.html)) is a great resource that provides thorough descriptions of critical processes, including specific artifacts to check for during your analysis. We based some of the following facts on Patrick’s article:

---

- `Idle` and `System`: These are not real processes (in the sense that they have no corresponding executable on disk). `Idle` is just a container that the kernel uses to charge CPU time for idle threads. Similarly, `System` serves as the default home for threads that run in kernel mode. Thus, the `System` process (PID 4) appears to own any sockets or handles to files that kernel modules open.
- `csrss.exe`: The client/server runtime subsystem plays a role in creating and deleting processes and threads. It maintains a private list of the objects that you can use to cross-reference with other data sources. On systems before Windows 7, this process also served as the broker of commands executed via `cmd.exe`, so you can extract command history from its memory space. Expect to see multiple CSRSS processes because each session gets a dedicated copy; however, watch out for attempts to exploit the naming convention (`csrsss.exe` or `cssrs.exe`). The real one is located in the system32 directory.
- `services.exe`: The Service Control Manager (SCM) is described more thoroughly in Chapter 12, but in short, it manages Windows services and maintains a list of such services in its private memory space. This process should be the parent for any `svchost.exe` (service host) instances that you see, in addition to processes such as `spoolsv.exe` and `SearchIndexer.exe` that implement services. There should be only one copy of `services.exe` on a system, and it should be running from the system32 directory.
- `svchost.exe`: A clean system has multiple shared host processes running concurrently, each providing a container for DLLs that implement services. As previously mentioned, their parent should be `services.exe`, and the path to their executable should point to the system32 directory. In his blog, Patrick identifies a few of the common names (such as `scvhost.exe` and `svch0st.exe`) used by malware to blend in with these processes.
- `lsass.exe`: The local security authority subsystem process is responsible for enforcing the security policy, verifying passwords, and creating access tokens. As such, it’s often the target of code injection because the plaintext password hashes can be found in its private memory space. There should be only one instance of `lsass.exe` running from the system32 directory, and its parent is `winlogon.exe` on pre-Vista machines, and `wininit.exe` on Vista and later systems. Stuxnet created two fake copies of `lsass.exe`, which caused them to stick out like a sore thumb.
- `winlogon.exe`: This process presents the interactive logon prompt, initiates the screen saver when necessary, helps load user profiles, and responds to Secure Attention Sequence (SAS) keyboard operations such as `CTRL+ALT+DEL`. Also, this process monitors files and directories for changes on systems that implement Windows File Protection (WFP). As with most other critical processes, its executable is located in the system32 directory.
- `explorer.exe`: You’ll see one Windows Explorer process for each logged-on user. It is responsible for handling a variety of user interactions such as GUI-based folder navigation, presenting the start menu, and so on. It also has access to sensitive material such as the documents you open and credentials you use to log in to FTP sites via Windows Explorer.
- `smss.exe`: The session manager is the first real user-mode process that starts during the boot sequence. It is responsible for creating the sessions (see Chapter 14) that isolate OS services from the various users who may log on via the console or Remote Desktop Protocol (RDP).

Although this list isn’t comprehensive, it should provide enough information to get you started. You should also become familiar with a number of noncritical (but common) processes, such as `IEXPLORE.EXE` (and other browsers), e-mail clients, chat clients, document readers (Word, Excel, Adobe), antivirus applications, disk encryption tools, remote access and file transfer utilities (SSH, Telnet, RDP, VNC), password-cracking tools, and exploit toolkits.

### Analyzing Process Activity

Volatility provides a few commands you can use for extracting information about processes:

- `pslist` finds and walks the doubly linked list of processes and prints a summary of the data. This method typically cannot show you terminated or hidden processes.
- `pstree` takes the output from `pslist` and formats it in a tree view, so you can easily see parent and child relationships.
- `psscan` scans for `_EPROCESS` objects instead of relying on the linked list. This plugin can also find terminated and unlinked (hidden) processes.
- `psxview` locates processes using alternate process listings, so you can then cross-reference different sources of information and reveal malicious discrepancies.

An example of the `pslist` command follows:

```
$ python vol.py -f lab.mem --profile=WinXPSP3x86 pslist
Volatility Foundation Volatility Framework 2.4
Offset(V)  Name              PID  PPID  Thds    Hnds  Sess  Start
---------- -------------- ------ ----- ----- ------- -----  -------------------
0x823c8830 System              4     0    56     537 ----- 
0x81e7e180 smss.exe          580     4     3      19 -----  2013-03-14 03:02:22
0x82315da0 csrss.exe         644   580    10     449     0  2013-03-14 03:02:25
0x81f37948 winlogon.exe      668   580    18     515     0  2013-03-14 03:02:26
0x81fec128 services.exe      712   668    15     281     0  2013-03-14 03:02:27
[snip]
0x81eb4300 vmtoolsd.exe     1684  1300     6     213     0  2013-03-14 03:02:45
0x8210b9c8 IEXPLORE.EXE     1764  1300    16     642     0  2013-03-14 03:03:04
0x81e79020 firefox.exe       180  1300    27     447     0  2013-03-14 03:03:05
0x81cb63d0 wuauclt.exe      1576  1072     3     104     0  2013-03-14 03:03:40
0x81e86bf8 alg.exe          1836   712     5     102     0  2013-03-14 03:04:00
0x8209eda0 wscntfy.exe      2672  1072     1      28     0  2013-03-14 03:04:01
0x82013340 jucheck.exe      2388  1656     2     104     0  2013-03-14 03:07:45
0x81e79418 thunderbird.exe  3832  1300    30     339     0  2013-03-14 03:12:54
0x8202b398 AcroRd32.exe     3684   180     0 -------     0  2013-03-14 14:19:16
0x81ecd3c0 cmd.exe          3812  3684     1      33     0  2013-03-14 14:19:29
0x81f55bd0 a[1].php         2280  3812     1     139     0  2013-03-14 14:19:30
0x8223b738 IEXPLORE.EXE     2276  2280     7     280     0  2013-03-14 14:19:32
0x822c8a58 AcroRd32.exe     2644   180     0 -------     0  2013-03-14 14:40:16
```

The first column in the output, `Offset(V)`, displays the virtual address (in kernel memory) of the `_EPROCESS` structure. Moving to the right, you see the process name (or at least the first 16 characters), its PID, parent PID, number of threads, number of handles, session ID, and create time. You can gather a number of interesting facts by looking at this data:

- Three browsers are running (two instances of `IEXPLORE.EXE` and one `firefox.exe`), an e-mail client (`thunderbird.exe`), and Adobe Reader (`AcroRd32.exe`). Thus, this machine is very likely to be a client or workstation, as opposed to a server. Furthermore, if you suspect a client-side attack vector (such as a drive-by download or phishing exploit), it is wise to mine these processes for data related to the incident because there’s a good chance that one or more of them was involved.
- All processes, including the system-critical ones, are running in session 0, which indicates this is an older (Windows XP or 2003) machine (that is, before session 0 isolation) and that only one user is currently logged on.
- Two of the `AcroRd32.exe` processes have 0 threads and an invalid handle table pointer (indicated by the dashed lines). If the exit time column were displayed (we truncated it to prevent lines from wrapping on the page), you’d see that these two processes have actually terminated. They’re “stuck” in the active process list because another process has an open handle to them (see *The Mis-leading Active in PsActiveProcessHead*: [http://mnin.blogspot.com/2011/03/mis-leading-active-in.html](http://mnin.blogspot.com/2011/03/mis-leading-active-in.html)).
- The process with PID 2280 (`a[1].php`) has an invalid extension for executables—it claims to be a PHP file. Furthermore, based on its creation time, it has a temporal relationship with several other processes that started during the same minute (14:19:XX), including a command shell (`cmd.exe`).

Just looking at the process list can often give you some immediate clues worthy of further investigation. The `pstree` plugin can extend your knowledge by providing a visual interpretation of the parent and child relationships. As shown in the following output, a process’ children are indented to the right and prepended with periods.

```
$ python vol.py -f lab.mem --profile=WinXPSP3x86 pstree
Volatility Foundation Volatility Framework 2.4

[snip]

0x82263378:explorer.exe           1300   1188     11    363 2013-03-14 03:02:42 
. 0x81e85da0:TSVNCache.exe        1556   1300      7     53 2013-03-14 03:02:43 
. 0x81e79020:firefox.exe           180   1300     27    447 2013-03-14 03:03:05 
.. 0x8202b398:AcroRd32.exe        3684    180      0 ------ 2013-03-14 14:19:16 
... 0x81ecd3c0:cmd.exe            3812   3684      1     33 2013-03-14 14:19:29
.... 0x81f55bd0:a[1].php          2280   3812      1    139 2013-03-14 14:19:30
..... 0x8223b738:IEXPLORE.EXE     2276   2280      7    280 2013-03-14 14:19:32
.. 0x822c8a58:AcroRd32.exe        2644    180      0 ------ 2013-03-14 14:40:16 
. 0x81e79418:thunderbird.exe      3832   1300     30    339 2013-03-14 03:12:54 
. 0x8210b9c8:IEXPLORE.EXE         1764   1300     16    642 2013-03-14 03:03:04
```

When viewing the processes as a tree, it’s much easier to determine the possible events that took place during the attack. You can see that `firefox.exe` (PID 180) was started by `explorer.exe` (PID 1300). This is normal—anytime you launch an application via the start menu or by double-clicking a desktop icon, the parent is Windows Explorer. It is also fairly common for browsers to create instances of Adobe Reader (`AcroRd32.exe`) to render PDF documents accessed via the web. The situation gets interesting when you see that `AcroRd32.exe` invoked a command shell (`cmd.exe`), which then started `a[1].php`.

At this point, you can assume that a web page visited with Firefox caused the browser to open a malicious PDF. An exploited flaw in `AcroRd32.exe` allowed the attacker to use a command shell to further his efforts in installing additional malware on the system.

### Process Tree Visualizations

Another way to visualize the parent and child relationships between processes is to use the `psscan` command with the dot graph renderer (`--output=dot`). This functionality is based on Andreas Schuster’s PTFinder tool ([http://www.dfrws.org/2006/proceedings/2-Schuster-pres.pdf](http://www.dfrws.org/2006/proceedings/2-Schuster-pres.pdf)), which also produced graphs for visual analysis. Because this perspective of the processes is based on pool-scanning through the physical address space, it also incorporates terminated and hidden processes into the graph. You can generate a graph like this:

```
$ python vol.py psscan –f memory.bin --profile=Win7SP1x64
       --output=dot 
       --output-file=processes.dot
```

Then open the output file in Graphviz ([http://www.graphviz.org](http://www.graphviz.org)), as shown in [Figure 6-3](#figure6-3).

Based on the graph, you can verify several statements that were previously discussed. For example, `System` starts `smss.exe`, which starts `csrss.exe` and `winlogon.exe` (on Windows XP and 2003 systems). You can then see `winlogon.exe` creating `services.exe` and `lsass.exe`. The SCM process goes on to create `spoolsv.exe` and various instances of `svchost.exe`. But this is only part of the picture. The remainder of the graph is shown in [Figure 6-4](#figure6-4).

As seen here, a process with PID 1188 started `explorer.exe`, but its `_EPROCESS` is no longer memory-resident, so additional information, such as the parent process name, is not available. This is typical of XP and 2003 systems because `userinit.exe` starts Explorer and it exits soon after. As you follow the remaining arrows, the diagram confirms the theories we generated by looking at the `pstree` output.

![c06f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f003.png)

*[**Figure 6-3:**](#figureanchor6-3) A graph generated by psscan that shows critical system process relationships*

![c06f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f004.png)

*[**Figure 6-4:**](#figureanchor6-4) A diagram of processes involved in a malicious PDF exploit delivered via the web*

### Detecting DKOM Attacks

Many attacks are possible with Direct Kernel Object Manipulation (DKOM), but one of the most common is hiding a process by unlinking its entry from the doubly linked list. To accomplish this, overwrite the `Flink` and `Blink` pointers of surrounding objects so that they point *around* the `_EPROCESS` structure of the process to hide. Tools that execute on a running system *and* Volatility’s `pslist` command are susceptible to this attack, because they rely on the linked list. However, the `psscan` plugin uses the pool-scanning approach described in Chapter 5. This way, you can find `_EPROCESS` objects in memory, even if they are unlinked from the list.

Before we begin with the example, consider the following ways that malware can directly modify kernel objects:

- By loading a kernel driver, which then has unrestricted access to objects in kernel memory
- By mapping a writable view of the `\Device\PhysicalMemory` object (however, starting with Windows 2003 SP1 and Vista, access to this object is restricted from user-mode programs)
- By using a special native API function called `ZwSystemDebugControl`

#### The Case of Prolaco

To demonstrate how you can use `psscan` to find hidden processes, we’ll focus on a malware sample known to antivirus vendors as Prolaco (see [https://www.avira.com/en/support-threats-description/tid/5377/](https://www.avira.com/en/support-threats-description/tid/5377/)). This malware performs DKOM entirely from user mode, without loading any kernel drivers. It does so by using the `ZwSystemDebugControl` API in almost the exact manner described by Alex Ionescu on the OpenRCE website ([http://www.openrce.org/blog/view/354/](http://www.openrce.org/blog/view/354/)). [Figure 6-5](#figure6-5) shows a decompilation of Prolaco, as produced by IDA Pro and Hex-Rays.

Based on the image, you can make the following conclusions about how the malware performs DKOM:

- It enables the debug privilege (`SeDebugPrivilege`), which gives the process the required access for using `ZwSystemDebugControl`.
- It calls `NtQuerySystemInformation` with the `SystemModuleInformation` class to locate the base address of the NT kernel module.
- It finds `PsInitialSystemProcess`, which is a global variable exported by the NT module that points to the first process’ `_EPROCESS` object.
- It walks the linked list of `_EPROCESS` objects until it finds the process with a PID that matches `PidOfProcessToHide`. The fixed number `0x88` being used inside the `while`loop is the offset to `ActiveProcessLinks` within the `_EPROCESS` structure for 32-bit Windows XP systems. Also note that `PidOfProcessToHide` is passed into the function as a parameter. The malware derives the value using `GetCurrentProcessId`, which means it tries to hide itself.
- It calls `WriteKernelMemory`, which is merely a wrapper around `ZwSystemDebugControl` that writes four bytes at a time to a specified address in kernel memory. The function is called once to overwrite the `Flink` pointer and once for the `Blink` pointer. [Figure 6-6](#figure6-6) shows the contents of this function.

![c06f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f005.png)

*[**Figure 6-5:**](#figureanchor6-5) Prolaco sample loaded in IDA with Hex-Rays, showing the main DKOM function*

![c06f006.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f006.png)

*[**Figure 6-6:**](#figureanchor6-6) The ZwSystemDebugControl call used to overwrite pointers in kernel memory*

At this point, all the aforementioned live system APIs (and tools that rely on them) will report inaccurate process listings. In particular, they’ll fail to identify the one process most relevant to your investigation: the malware that just unlinked its own `_EPROCESS`.

#### Alternate Process Listings

As previously mentioned, it’s never a good idea to base your conclusions on only one piece of evidence. Because the process list can be manipulated, you should be aware of the available backup methods, or alternate process listings. Here are a few of them:

- **Process object scanning:** This is the pool-scanning approach discussed in Chapter 5. Remember that the pool tags it finds are nonessential; thus, they can also be manipulated to evade the scanner.
- **Thread scanning:** Because every process *must* have at least one active thread, you can scan for `_ETHREAD` objects and then map them back to their owning process. The member used for mapping is either `_ETHREAD.ThreadsProcess` (Windows XP and 2003) or `_ETHREAD.Tcb.Process` (Windows Vista and later). Thus, even if a rootkit manipulated the process’ pool tags to hide from `psscan`, it would also need to go back and modify the pool tags for all the process’ threads.
- **CSRSS handle table:** As discussed in the critical system process descriptions, `csrss.exe` is involved in the creation of every process and thread (with the exception of itself and the processes that started before it). Thus, you can walk this process’ handle table, as described later in the chapter, and identify all `_EPROCESS` objects that way.
- **PspCid table:** This is a special handle table located in kernel memory that stores a reference to all active process and thread objects. The `PspCidTable` member of the kernel debugger data structure points to the table. Two rootkit detection tools, Blacklight and IceSword, relied on the PspCid table to find hidden processes. However, the author of FUTo (see [http://www.openrce.org/articles/full_view/19](http://www.openrce.org/articles/full_view/19)) proved it was still possible to hide by removing processes from the table.
- **Session processes:** The `SessionProcessLinks` member of `_EPROCESS` associates all processes that belong to a particular user’s logon session. It’s not any harder to unlink a process from this list, as opposed to the `ActiveProcessLinks` list. But because live system APIs don’t depend on it, attackers rarely find value in targeting it.
- **Desktop threads:** One of the structures discussed in Chapter 14 is the Desktop (`tagDESKTOP`). These structures store a list of all threads attached to each desktop, and you can easily map a thread back to its owning process.

Plenty of additional sources of process listings remain. However, we’ve never encountered a rootkit that even comes close to hiding from the `pslist` plugin and then hides from all these methods as well. That’s why we built the `psxview` plugin, described next.

#### Process Cross-View Plugin

The `psxview` plugin enumerates processes in seven different ways: the active process linked list and the six methods previously identified. Thus, it’s unlikely that a rootkit can successfully hide from `psxview`. Realistically, it’s far easier to just inject code into a process that’s not hidden than to hide a process seven different ways in a reliable (that is, no bugs) and portable manner (works across all Windows versions).

The `psxview` plugin displays one row for each process it finds and seven columns that contain `True` or `False`, based on whether the process was found using the respective method. If you supply the `--apply-rules` option, you might also see `Okay` in the columns, which indicates that although the process was *not* found, it meets one of the valid exceptions described in the following list:

- Processes that start before `csrss.exe` (including `System`, `smss.exe` and `csrss.exe` itself) are not in the CSRSS handle table.
- Processes that start before `smss.exe` (including `System` and `smss.exe`) are not in the session process or desktop thread lists.
- Processes that have exited will not be found by any of the methods except process object scanning and thread scanning (if an `_EPROCESS` or `_ETHREAD` still happens to be memory resident).

An example of the `psxview` output when used with `--apply-rules` is shown in [Figure 6-7](#figure6-7). The last two processes (`msiexec.exe` and `rundll32.exe`) are found only by the process object scanner. However, as shown in the far right column, they both have nonzero exit times, which means they have terminated.

---

**Warning**

After attackers gain access to kernel memory, they can manipulate anything they want. In this case, they could overwrite the `_EPROCESS.ExitTime` member to make it appear as if the process exited; thus the `--apply-rules` option would improperly report it as `Okay`. However, processes that have truly exited have zero threads and an invalid handle table—so you can always double-check what those fields contain.

---

![c06f007.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f007.png)

*[**Figure 6-7:**](#figureanchor6-7) Psxview output after applying rules to consider exceptions*

As shown in the figure, the only process that stands out after considering the rules is `1_doc_RCData_61.exe`—it’s found by every method *except* the linked list of active processes. This is a clear indication that it is trying to hide from live system APIs by unlinking from the list.

## Process Tokens

A process’ token describes its security context. This context includes security identifiers (SIDs) of users or groups that the process is running as and the various privileges (specific tasks) that it is allowed to perform. When the kernel needs to decide whether a process can access an object or call a particular API, it consults data in the process’ token. As a result, this structure dictates many of the security-related controls that involve processes. This section describes how to leverage tokens to augment your investigations.

---

## **Analysis Objectives**

Your objectives are these:

- **Map SIDs to usernames:** A process’ token contains numerical SID values that you can translate into a string and then resolve into a user or group name. This ultimately enables you to determine the primary user account under which a process is running.
- **Detect lateral movement:** When hacking techniques such as pass-the-hash ([http://www.microsoft.com/security/sir/strategy/default.aspx#!pass_the_hash_attacks](http://www.microsoft.com/security/sir/strategy/default.aspx#!pass_the_hash_attacks)) are successful, they leave obvious artifacts in a process’ token. Specifically, you see a process’ security context jump to that of Domain Admin or Enterprise Admin.
- **Profile process behaviors:** A privilege (discussed later in the chapter) is the right to perform a specific task. If a process plans to engage in a task, it first must ensure that the privilege is present and enabled in its token. Thus, an *ex post facto* analysis of the privileges that a process has acquired can provide clues about what the process did (or planned to do).
- **Detect privilege escalation:** Some attacks have proven that live tools such as Process Explorer can be deceived into reporting that a process has fewer privileges than it actually has. You can use memory forensics to more accurately determine the truth.

## **Data Structures**

The `_TOKEN` structure is large, so we won’t display all the members. Furthermore, it changed significantly with respect to how privilege information is stored between Windows 2003 and Vista. The following code first shows the earlier versions of the structures, from a 64-bit 2003 system:

```
>>> dt("_TOKEN")
'_TOKEN' (208 bytes)
0x0   : TokenSource         ['_TOKEN_SOURCE']
0x10  : TokenId             ['_LUID']
0x18  : AuthenticationId    ['_LUID']
[snip]
0x4c  : UserAndGroupCount   ['unsigned long']
0x50  : RestrictedSidCount  ['unsigned long']
0x54  : PrivilegeCount      ['unsigned long']
[snip]
0x68  : UserAndGroups       ['pointer', ['array', 
    lambda x: x.UserAndGroupCount, ['_SID_AND_ATTRIBUTES']]]
0x70  : RestrictedSids      ['pointer64', ['_SID_AND_ATTRIBUTES']]
0x78  : PrimaryGroup        ['pointer64', ['void']]
0x80  : Privileges          ['pointer', ['array', 
    lambda x: x.PrivilegeCount, ['_LUID_AND_ATTRIBUTES']]]

>>> dt("_SID_AND_ATTRIBUTES")
'_SID_AND_ATTRIBUTES' (16 bytes)
0x0   : Sid                 ['pointer64', ['void']]
0x8   : Attributes          ['unsigned long']

>>> dt("_SID")
'_SID' (12 bytes)
0x0   : Revision              ['unsigned char']
0x1   : SubAuthorityCount     ['unsigned char']
0x2   : IdentifierAuthority   ['_SID_IDENTIFIER_AUTHORITY']
0x8   : SubAuthority          ['array', 
    lambda x: x.SubAuthorityCount, ['unsigned long']]

>>> dt("_SID_IDENTIFIER_AUTHORITY")
'_SID_IDENTIFIER_AUTHORITY' (6 bytes)
0x0   : Value                          ['array', 6, ['unsigned char']]

>>> dt("_LUID_AND_ATTRIBUTES")
'_LUID_AND_ATTRIBUTES' (12 bytes)
0x0   : Luid                           ['_LUID']
0x8   : Attributes                     ['unsigned long']

>>> dt("_LUID")
'_LUID' (8 bytes)
0x0   : LowPart                        ['unsigned long']
0x4   : HighPart                       ['long']
```

Here are the equivalent structures for 64-bit Windows 7:

```
>>> dt("_TOKEN")
'_TOKEN' (784 bytes)
0x0   : TokenSource         ['_TOKEN_SOURCE']
0x10  : TokenId             ['_LUID']
0x18  : AuthenticationId    ['_LUID']
[snip]
0x40  : Privileges          ['_SEP_TOKEN_PRIVILEGES']
0x58  : AuditPolicy         ['_SEP_AUDIT_POLICY']
0x74  : SessionId           ['unsigned long']
0x78  : UserAndGroupCount   ['unsigned long']
[snip]
0x90  : UserAndGroups       ['pointer', ['array', 
   lambda x: x.UserAndGroupCount, ['_SID_AND_ATTRIBUTES']]]

>>> dt("_SEP_TOKEN_PRIVILEGES")
'_SEP_TOKEN_PRIVILEGES' (24 bytes)
0x0   : Present                        ['unsigned long long']
0x8   : Enabled                        ['unsigned long long']
0x10  : EnabledByDefault               ['unsigned long long']
```

## **Key Points**

The key points are these:

- `UserAndGroupCount`: This integer stores the size of the `UserAndGroups` array.
- `UserAndGroups`: An array of `_SID_AND_ATTRIBUTES` structures associated with the token. Each element in the array describes a different user or group that the process is a member of. The `Sid` member of `_SID_AND_ATTRIBUTES` points to a `_SID` structure, which has `IdentifierAuthority` and `SubAuthority` members that you can combine to form the `S-1-5-[snip]` SID strings.
- PrivilegeCount (Windows XP and 2003 only): This integer stores the size of the `Privileges` array.
- `Privileges` (Windows XP and 2003): An array of `_LUID_AND_ATTRIBUTES` structures that each describe a different privilege and its attributes (that is, present, enabled, enabled by default).
- `Privileges` (Windows Vista and later): This is an instance of `_SEP_TOKEN_PRIVILEGES`, which has three parallel 64-bit values (`Present`, `Enabled`, `EnabledByDefault`). The bit positions correspond to particular privileges, and the values of the bit (`on` or `off`) describe the privilege’s status.

---

### Live Response: Accessing Tokens

On a live machine, a process can access its own token through the `OpenProcessToken` API. To enumerate the SIDs or privileges, it can then use `GetTokenInformation` with the desired parameters. With administrator access, it can also query (or set) the tokens of other users’ processes, including the system-critical ones. Of course, existing tools already provide this type of functionality for you, such as Sysinternals Process Explorer. [Figure 6-8](#figure6-8) shows the token information for `explorer.exe`. The SID data appears at the top, and the privilege data appears in the lower level.

![c06f008.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f008.png)

*[**Figure 6-8:**](#figureanchor6-8) The Process Explorer Security tab shows SID and privilege information from process tokens.*

This instance of `explorer.exe` belongs to a user named Jimmy, whose SID string is `S-1-5-21-[snip]-1000`. By analyzing the other SIDs in this process’ token, you can see it’s also in the `Everyone`, `LOCAL`, and `NT AUTHORITY\Authenticated Users` groups. Five privileges are also present in the token, but only one of them is currently enabled. The next few pages discuss these concepts in more detail.

### Extracting and Translating SIDs in Memory

The live APIs we just described are convenient ways to enumerate SIDs on running systems. Windows also provides the `ConvertSidToStringSid` API that translates the numerical data in a `_SID` structure to the human readable `S-1-5-[snip]` format. It also provides `LookupAccountSid` that returns an account name for a given SID. However, because we’re dealing with memory dumps, Volatility (the `getsids` plugin in particular) is responsible for finding each process’ token, extracting the numerical components of the `_SID` structure, and translating them into strings. After it is done, it maps the strings to user and group names on the local computer or domain.

You can perform the mapping in a few different ways. First, the Known SIDs (see [http://support.microsoft.com/kb/243330](http://support.microsoft.com/kb/243330)) are hardcoded into Windows and thus can be hardcoded into the Volatility plugin. They consist of SIDs such as `S-1-5` (NT Authority) and `S-1-5-32-544` (Administrators). There are also Service SIDs prefixed with `S-1-5-80`. The remainder of the SID in this case is composed of the SHA1 hash of the corresponding service name (in Unicode, uppercase)—an algorithm described in greater detail here: [http://volatility-labs.blogspot.com/2012/09/movp-23-event-logs-and-service-sids.html](http://volatility-labs.blogspot.com/2012/09/movp-23-event-logs-and-service-sids.html).

Finally, there are User SIDs such as `S-1-5-21-4010035002-774237572-2085959976-1000`. These SIDs break down into the following components:

- **S:** Prefix indicating that the string is a SID
- **1:** The revision level (version of the SID specification) from `_SID.Revision`
- **5:** The identifier authority value from `_SID.IdentifierAuthority.Value`
- **21-4010035002-774237572-2085959976:** The local computer or domain identifier from the `_SID.SubAuthority` values
- **1000:** A relative identifier that represents any user or group that doesn’t exist by default

You can map the SID string to a username by querying the registry. The following command shows an example of how to do this:

```
$ python vol.py -f memory.img --profile=Win7SP0x86 printkey -K "Microsoft\Windows
NT\CurrentVersion\ProfileList\S-1-5-21-4010035002-774237572-2085959976-1000"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile
----------------------------
Registry: User Specified
Key name: S-1-5-21-4010035002-774237572-2085959976-1000 (S)
Last updated: 2011-06-09 19:50:32 

Subkeys:

Values:
REG_EXPAND_SZ ProfileImagePath : (S) C:\Users\nkESis3ns88S
REG_DWORD     Flags           : (S) 0
REG_DWORD     State           : (S) 0
REG_BINARY    Sid             : (S) 
0x00000000  01 05 00 00 00 00 00 05 15 00 00 00 3a 47 04 ef   ............:G..
0x00000010  84 ed 25 2e 28 39 55 7c e8 03 00 00               ..%.(9U|....
[snip]
```

By appending the SID string to the `ProfileList` registry key, you can see a value named `ProfileImagePath`. The username is then defined within the profile path. In this case, the user’s name was `nkESis3ns88S` (it was a randomly generated backdoor account an attacker created to retain access to the system).

---

**NOTE**

For more information on translating SID values, see the following links:

- *Linking Processes to Users:*[http://moyix.blogspot.com/2008/08/linking-processes-to-users.html](http://moyix.blogspot.com/2008/08/linking-processes-to-users.html).
- *How to Associate a Username with a Security Identifier (SID):*[http://support.microsoft.com/kb/154599/en-us](http://support.microsoft.com/kb/154599/en-us)
- *Security Identifier Structure:*[http://technet.microsoft.com/en-us/library/cc962011.aspx](http://technet.microsoft.com/en-us/library/cc962011.aspx).

---

### Detecting Lateral Movement

If you need to associate a process with a user account or investigate potential lateral movement attempts, use the `getsids` plugin. The following output is from Jack Crook’s GrrCON forensic challenge (see [http://michsec.org/2012/09/misec-meetup-october-2012/](http://michsec.org/2012/09/misec-meetup-october-2012/)).

```
$ python vol.py –f grrcon.img --profile=WinXPSP3x86 getsids –p 1096
Volatility Foundation Volatility Framework 2.4

explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-500 (administrator)
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-513 (Domain Users)
explorer.exe: S-1-1-0 (Everyone)
explorer.exe: S-1-5-32-545 (Users)
explorer.exe: S-1-5-32-544 (Administrators)
explorer.exe: S-1-5-4 (Interactive)
explorer.exe: S-1-5-11 (Authenticated Users)
explorer.exe: S-1-5-5-0-206541 (Logon Session)
explorer.exe: S-1-2-0 (Local (Users with the ability to log in locally))
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-519 (Enterprise Admins)
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-1115
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-518 (Schema Admins)
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-512 (Domain Admins)
```

This command shows the SIDs associated with `explorer.exe` for the current logged-on user. You’ll immediately notice that one SID (`S-1-5-21-[snip]-1115`) doesn’t display an account name. On systems that don’t authenticate to a domain, you’ll see the local user’s name next to the SID. In this case, however, because Volatility doesn’t have access to the remote machine’s registry (that is, the domain controller or Active Directory server), it cannot perform the resolution.

The question you can answer with `getsids` is this: What level of access did the attacker gain? Don’t stop short after seeing that `explorer.exe` is a member of the Administrators group. The attacker on this machine actually joined the Domain and Enterprise Admins groups, allowing him to move laterally throughout the entire corporate network. In this particular scenario, the attacker combined a Poison Ivy (PI) Remote Access Trojan (RAT) with the use of a Pass the Hash (PtH) attack. You can read a full analysis of this attack here: [http://volatility-labs.blogspot.com/2012/10/solving-grrcon-network-forensics.html](http://volatility-labs.blogspot.com/2012/10/solving-grrcon-network-forensics.html).

## Privileges

Privileges are another critical component involved in security and access control. A privilege is the permission to perform a specific task, such as debugging a process, shutting down the computer, changing the time zone, or loading a kernel driver. Before a process can enable a privilege, the privilege must be present in the process’ token. Administrators decide which privileges are present by configuring them in the Local Security Policy (LSP), as shown in [Figure 6-9](#figure6-9), or programmatically by calling `LsaAddAccountRights`. You can access the LSP by going to Start ⇒ Run and typing `SecPol.msc`.

![c06f009.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f009.png)

*[**Figure 6-9:**](#figureanchor6-9) Administrators can configure privileges using the Local Security Policy editor.*

### Commonly Exploited Privileges

After a privilege is present in a process’ token, it must be enabled. The following list describes a few ways to enable privileges:

- **Enabled by default:** The LSP can specify that privileges be enabled by default when a process starts.
- **Inheritance:** Unless otherwise specified, child processes inherit the security context of their creator (parent).
- **Explicit enabling:** A process can explicitly enable a privilege using the `AdjustTokenPrivileges` API.

From a forensic perspective, you should be most concerned with the following privileges when they’ve been *explicitly* enabled. For a full list of possible privileges and their descriptions, see [http://msdn.microsoft.com/en-us/library/windows/desktop/bb530716(v=vs.85).aspx](http://msdn.microsoft.com/en-us/library/windows/desktop/bb530716(v=vs.85).aspx).

- `SeBackupPrivilege`: This grants read access to any file on the file system, regardless of its specified access control list (ACL). Attackers can leverage this privilege to copy locked files.
- `SeDebugPrivilege`: This grants the ability to read from or write to another process’ private memory space. It allows malware to bypass the security boundaries that typically isolate processes. Practically all malware that performs code injection from user mode relies on enabling this privilege.
- `SeLoadDriverPrivilege`: This grants the ability to load or unload kernel drivers.
- `SeChangeNotifyPrivilege`: This allows the caller to register a callback function that gets executed when specific files and directories change. Attackers can use this to determine immediately when one of their configuration or executable files are removed by antivirus or administrators.
- `SeShutdownPrivilege`: This allows the caller to reboot or shut down the system. Some infections, such as those that modify the Master Boot Record (MBR) don’t activate until the next time the system boots. Thus, you’ll often see malware trying to manually speed up the procedure by invoking a reboot. **NOTE** Cem Gurkok helped design the Volatility support for analyzing privileges in memory. You can read his presentation, *Reverse Engineering with Volatility on a Live System: The Analysis of Process Token Privileges*, here: [http://volatility-labs.blogspot.com/2012/10/omfw-2012-analysis-of-process-token.html](http://volatility-labs.blogspot.com/2012/10/omfw-2012-analysis-of-process-token.html).

### Analyzing Explicit Privileges

The reason why we’re typically most interested in explicitly enabled privileges is because it shows awareness, or intent. If a process can change the time zone, because the LSP gives *all* processes that capability or because its parent process had that capability, that doesn’t really tell you anything about the intended functionality of the process. On the other hand, if a process explicitly enabled the privilege to change time zones, you can bet it will try changing the time zone.

Here’s the output of the Volatility `privs` plugin. You’ll see the privilege name along with its attributes (present, enabled, and/or enabled by default).

```
$ python vol.py -f grrcon.img privs -p 1096
Volatility Foundation Volatility Framework 2.4
Pid    Process       Privilege                       Attributes               
------ ------------- ------------------------------- ------------------------
1096   explorer.exe  SeChangeNotifyPrivilege         Present,Enabled,Default  
1096   explorer.exe  SeShutdownPrivilege             Present                  
1096   explorer.exe  SeUndockPrivilege               Present,Enabled          
1096   explorer.exe  SeSecurityPrivilege             Present                  
1096   explorer.exe  SeBackupPrivilege               Present                  
1096   explorer.exe  SeRestorePrivilege              Present                  
1096   explorer.exe  SeSystemtimePrivilege           Present                  
1096   explorer.exe  SeRemoteShutdownPrivilege       Present                  
1096   explorer.exe  SeTakeOwnershipPrivilege        Present                  
1096   explorer.exe  SeDebugPrivilege                Present,Enabled          
1096   explorer.exe  SeSystemEnvironmentPrivilege    Present                  
1096   explorer.exe  SeSystemProfilePrivilege        Present                  
1096   explorer.exe  SeProfileSingleProcessPrivilege Present                  
1096   explorer.exe  SeIncreaseBasePriorityPrivilege Present                  
1096   explorer.exe  SeLoadDriverPrivilege           Present,Enabled          
1096   explorer.exe  SeCreatePagefilePrivilege       Present                  
1096   explorer.exe  SeIncreaseQuotaPrivilege        Present                  
1096   explorer.exe  SeManageVolumePrivilege         Present                  
1096   explorer.exe  SeCreateGlobalPrivilege         Present,Enabled,Default  
1096   explorer.exe  SeImpersonatePrivilege          Present,Enabled,Default  
```

You can see several privileges present in the output. Only six of them are enabled, but three are enabled by default. Thus, you can conclude that `explorer.exe` explicitly enabled the undock privilege, debug privilege, and load driver privilege. You’ll realize over time that `explorer.exe` always enables the undock privilege, so that one is not concerning. But why does Windows Explorer need to debug other processes and load kernel drivers? The answer is simple: It doesn’t! This process is hosting an injected Poison Ivy (PI) sample, and PI explicitly enabled the privileges.

### Detecting Token Manipulation

As previously mentioned, the Windows API (`AdjustTokenPrivileges`) does not, and *should* not, allow enabling a privilege that isn’t present in a token. Thus, it makes sense that APIs such as `GetTokenInformation` (and tools based on this API) would first check what’s present and then return the enabled subset. Here’s the catch: A talented researcher, Cesar Cerrudo, discovered that when checking to see if a process can perform a task, the kernel cares only about what’s enabled. As a result, Cesar proposed in his paper, *Easy Local Windows Kernel Exploitation* (see [https://www.blackhat.com/html/bh-us-12/bh-us-12-archives.html#Cerrudo](https://www.blackhat.com/html/bh-us-12/bh-us-12-archives.html#Cerrudo)), a method to bypass Windows APIs and enable all privileges for a process, even without them being present.

#### Attack Simulation with Volshell

Cesar’s attack is based on the DKOM approach. He locates the `_SEP_TOKEN_PRIVILEGES` structure for the target process and sets the 64-bit `Enabled` member to `0xFFFFFFFFFFFFFFFF`. This effectively enables all possible privileges. He does *not* update the `Present` member, so it will reflect only the privileges that were present before the attack. To simulate these steps, you can use Volatility in write mode to modify a VM’s memory. When you’re done, resume the VM, and the changes will take effect. This is easier than writing a kernel driver—especially when it’s just for testing purposes.

```
$ python vol.py -f VistaSP0x64.vmem --profile=VistaSP2x64 volshell --write
Volatility Foundation Volatility Framework 2.4
Write support requested.  Please type "Yes, I want to enable write support" 
Yes, I want to enable write support
Current context: process System, pid=4, ppid=0 DTB=0x124000
To get help, type 'hh()'
>>> cc(pid = 1824)
Current context: process explorer.exe, pid=1824, ppid=1668 DTB=0x918d000
```

Now that you’re in the target process’ context, obtain a pointer to its `_TOKEN` structure. Then you can print the 64-bit numbers as a binary string, like this:

```
>>> token = proc().get_token()
>>> bin(token.Privileges.Present)
'0b11000000010100010000000000000000000'
>>> bin(token.Privileges.Enabled)
'0b100000000000000000000000'
```

The next commands set all bits in the `Enabled` member and reprint the values to verify that it indeed has been updated. Then, you can quit the shell.

```
>>> token.Privileges.Enabled = 0xFFFFFFFFFFFFFFFF
>>> bin(token.Privileges.Present)
'0b11000000010100010000000000000000000'
>>> bin(token.Privileges.Enabled)
'0b1111111111111111111111111111111111111111111111111111111111111111'
>>> quit()
```

[Figure 6-10](#figure6-10) shows how the kernel data structure appears after the manipulation.

---

**NOTE**

[Figure 6-10](#figure6-10) is not drawn to scale (the members shown aren’t actually 64 bits wide). Also, if you want to see the exact bit position mappings, look in the `volatility/plugins/privileges.py` source file.

---

![c06f010.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f010.png)

*[**Figure 6-10:**](#figureanchor6-10) All bits in the Enabled member have been set by directly modifying the structure in kernel memory.*

According to the previous `volshell` output *and* [Figure 6-10](#figure6-10), only five bits are set in the `Present` member, which means that a maximum of five privileges are reported by tools running on the live system. [Figure 6-11](#figure6-11) shows how this appears in Process Explorer:

![c06f011.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f011.png)

*[**Figure 6-11:**](#figureanchor6-11) Process Explorer is tricked into reporting that only five privileges are enabled.*

As expected, it shows that only five privileges are enabled.

#### Revealing the Truth

However, if you analyze the VM memory dump using the `privs` plugin, which does *not* use the same logic as the Windows API, you’ll see that all privileges are enabled, yet many of them are not even present (which should never happen). Thus, `explorer.exe` can carry out any task it desires, and live tools on the system continue to report that those capabilities don’t exist.

```
$ python vol.py -f VistaSP0x64.vmem --profile=VistaSP2x64 privs -p 1824
Volatility Foundation Volatility Framework 2.4
Pid   Process       Privilege                        Attributes        
----- ------------  -------------------------------- --------------------
1824  explorer.exe  SeCreateTokenPrivilege           Enabled                  
1824  explorer.exe  SeAssignPrimaryTokenPrivilege    Enabled                  
[snip]             
1824  explorer.exe  SeRestorePrivilege               Enabled                  
1824  explorer.exe  SeShutdownPrivilege              Present,Enabled          
1824  explorer.exe  SeDebugPrivilege                 Enabled                  
1824  explorer.exe  SeAuditPrivilege                 Enabled                  
1824  explorer.exe  SeSystemEnvironmentPrivilege     Enabled                  
1824  explorer.exe  SeChangeNotifyPrivilege          Present,Enabled,Default  
1824  explorer.exe  SeRemoteShutdownPrivilege        Enabled                  
1824  explorer.exe  SeUndockPrivilege                Present,Enabled          
1824  explorer.exe  SeSyncAgentPrivilege             Enabled                  
[snip]            
1824  explorer.exe  SeRelabelPrivilege               Enabled                  
1824  explorer.exe  SeIncreaseWorkingSetPrivilege    Present,Enabled          
1824  explorer.exe  SeTimeZonePrivilege              Present,Enabled          
1824  explorer.exe  SeCreateSymbolicLinkPrivilege    Enabled   
```

One caveat to Cesar’s attack is that you need kernel-level access in the first place to modify the `_SEP_TOKEN_PRIVILEGES` structure. However, the point was to prolong access to a system by deceiving live tools and incident responders—not to exploit a privilege escalation vulnerability.

## Process Handles

A *handle* is a reference to an open instance of a kernel object, such as a file, registry key, mutex, process, or thread. As discussed in Chapter 5, there are close to 40 different types of kernel objects. By enumerating and analyzing the specific objects a process was accessing at the time of a memory capture, it is possible to arrive at a number of forensically relevant conclusions—such as what process was reading or writing a particular file, what process accessed one of the registry run keys, and which process mapped remote file systems.

### Lifetime of a Handle

Before a process can access an object, it first opens a handle to the object by calling an API such as `CreateFile`, `RegOpenKeyEx`, or `CreateMutex`. These APIs return a special Windows data type called `HANDLE`, which is simply an index into a process-specific handle table. For example, when you call `CreateFile`, a pointer to the corresponding `_FILE_OBJECT` in kernel memory is placed in the first available slot in the calling process’ handle table, and the respective index (such as `0x40`) is returned. Additionally, the handle count for the object is incremented. The calling process then passes the `HANDLE` value to functions that perform operations on the object, such as reading, writing, waiting, or deleting. Thus, APIs such as `ReadFile` and `WriteFile` work in the following manner:

1. Find the base address of the calling process’ handle table.
2. Seek to index `0x40`.
3. Retrieve the `_FILE_OBJECT` pointer.
4. Carry out the requested operation.

When a process is finished using an object, it should close the handle by calling the appropriate function (`CloseHandle`, `RegCloseHandle`, and so on). These APIs decrement the object’s handle count and remove the pointer to the object from the process’ handle table. At this point, the handle table index can be reused to store another type of object. However, the actual object (i.e., the `_FILE_OBJECT`) will not be freed or overwritten until the handle count reaches zero, which prevents one process from deleting an object that is currently in use by another process.

---

**NOTE**

The handle table model was designed for both convenience and security. It’s convenient because you don’t have to pass the full name or path of an object each time you perform an operation—only when you initially open or create the object. For security purposes, it also helps to conceal the addresses of objects in kernel memory. Because processes in user mode should never directly access kernel objects, there’s no reason why they would need the pointers. Furthermore, the model provides a centralized way for the kernel to monitor access to kernel objects, thus giving it the chance to enforce security based on SIDs and privileges.

---

### Reference Counts and Kernel Handles

So far in this section, we’ve been referring to processes as the entities that interact with objects via handles. However, kernel modules, or threads in kernel mode, can call the equivalent kernel APIs (i.e., `NtCreateFile`, `NtReadFile`, `NtCreateMutex`) in a similar manner. In this case, the handles are allocated from the `System` (PID 4) process’ handle table. Thus, when you dump the handles of the `System` process, you’re actually seeing all the currently open resources requested by kernel modules.

It’s also possible for code in the kernel to access existing objects directly, without first opening handles. For example, as long as the address of the object is known, you can use `ObReferenceObjectByPointer`. This API increments the reference count, rather than the handle count, so the OS will not delete the object while it’s still being referenced. Obviously, calling `ObDereferenceObject` is recommended, or else the objects may persist unnecessarily (this is known as a handle or reference leak). Although leaks are bad for performance, they’re good for forensics—it is like an attacker failing to clean up the crime scene.

In many cases, even if handles are closed, and references are released, there’s still a chance that you can find the objects by scanning the physical address space, as described in Chapter 5. Of course, they wouldn’t be associated with a process’ handle table at that point, but their presence in RAM can still lend clues to your investigations. Likewise, after a process terminates, its handle table is destroyed, but that doesn’t mean all objects created by the process are destroyed at the same time.

---

## **Analysis Objectives**

Your objectives are these:

- **Handle table internals:** Learning the internals of handles and handle tables can give you a greater understanding of the objects and artifacts you find in memory dumps.
- **Targeted object attribution**: Given an indicator such as a filename, registry key path, mutex, or other object—you can trace it back to the process, or processes responsible for creating or accessing it.
- **Open-ended investigations**: If you don’t have a specific list of initial indicators, you can still gain a great deal of knowledge about the behaviors and intentions of an unknown process by analyzing the contents of its handle table.
- **Detect registry persistence**: Learn how to analyze open registry handles to determine which keys a process uses to store its configuration or persistence data.
- **Identify remote mapped drives**: Adversaries frequently look for IP addresses and names of other computers in the workgroup or domain and then try to map them for remote read or write access. You’ll learn how to find evidence of exactly which systems and paths were accessed by looking in handle tables.

---

### Handle Table Internals

Each process’ `_EPROCESS.ObjectTable` member points to a handle table (`_HANDLE_TABLE`). This structure has a `TableCode` that serves two critical purposes: It specifies the number of levels in the table and it points to the base address of the first level. All processes start out with a single-level table, which is shown in [Figure 6-12](#figure6-12). The table size is one page (4096 bytes), and this scheme allows for up to 512 handles on a 32-bit system or 256 on a 64-bit system. Indexes in the table contain `_HANDLE_TABLE_ENTRY` structures if they’re in use; otherwise, they’re zeroed out.

The handle table entries contain an `Object` member that points to the `_OBJECT_HEADER` of the corresponding object. By navigating these fields, you can locate both the object name and the object body (i.e., `_FILE_OBJECT`, `_EPROCESS`).

Some processes require more open handles than the single-level table permits. Thus, Windows can expand on-demand to a scheme involving up to three levels. For example, in a two-level table, the first level is still a 4096-byte block of memory, but it is divided up into 1024 slots (32-bit) or 512 slots (64-bit). Each slot stores a pointer to an array of `_HANDLE_TABLE_ENTRY` structures. Thus, on a 32-bit platform, the two-level handle table can support up to 1024 * 512 = 524,288 handles.

![c06f012.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f012.png)

*[**Figure 6-12:**](#figureanchor6-12) A diagram of a single-level handle table*

Similarly, a three-level table on a 32-bit system can theoretically support 1024 * 1024 * 512 = 536,870,912 handles. However, as described in “Pushing the Limits of Windows: Handles” ([https://blogs.technet.com/b/markrussinovich/archive/2009/09/29/3283844.aspx](https://blogs.technet.com/b/markrussinovich/archive/2009/09/29/3283844.aspx)), the observed limits are actually far fewer. For one, the kernel enforces a hard-coded limit of about 16 million. Additionally, any process that requires more than several thousand open handles concurrently is probably experiencing a handle table leak (i.e., forgetting to close its handles). Thus, the hard-coded maximum serves as an early indicator of poorly written applications.

---

## **Data Structures**

The output that follows shows the handle table and handle table entry structure for 64-bit Windows 7 systems:

```
>>> dt("_HANDLE_TABLE")
'_HANDLE_TABLE' (104 bytes)
0x0   : TableCode                    ['unsigned long long']
0x8   : QuotaProcess                 ['pointer64', ['_EPROCESS']]
0x10  : UniqueProcessId              ['pointer64', ['void']]
0x18  : HandleLock                   ['_EX_PUSH_LOCK']
0x20  : HandleTableList              ['_LIST_ENTRY']
0x30  : HandleContentionEvent        ['_EX_PUSH_LOCK']
0x38  : DebugInfo                    ['pointer64', ['_HANDLE_TRACE_DEBUG_INFO']]
0x40  : ExtraInfoPages               ['long']
0x44  : Flags                        ['unsigned long']
0x44  : StrictFIFO                   ['BitField', 
   {'end_bit': 1, 'start_bit': 0, 'native_type': 'unsigned char'}]
0x48  : FirstFreeHandle              ['unsigned long']
0x50  : LastFreeHandleEntry          ['pointer64', ['_HANDLE_TABLE_ENTRY']]
0x58  : HandleCount                  ['unsigned long']
0x5c  : NextHandleNeedingPool        ['unsigned long']
0x60  : HandleCountHighWatermark     ['unsigned long']

>>> dt('_HANDLE_TABLE_ENTRY')
'_HANDLE_TABLE_ENTRY' (16 bytes)
0x0   : InfoTable                    ['pointer64', ['_HANDLE_TABLE_ENTRY_INFO']]
0x0   : ObAttributes                 ['unsigned long']
0x0   : Object                       ['_EX_FAST_REF']
0x0   : Value                        ['unsigned long long']
0x8   : GrantedAccess                ['unsigned long']
0x8   : GrantedAccessIndex           ['unsigned short']
0x8   : NextFreeTableEntry           ['unsigned long']
0xa   : CreatorBackTraceIndex        ['unsigned short']
```

## **Key Points**

The key points for `_HANDLE_TABLE` are these:

- `TableCode`: This value tells you the number of levels in the table and points to the address of the top-level table. The dual purpose is achieved using a bit mask of seven (7). For example, to obtain the number of tables, you can compute `TableCode & 7`; and to obtain the address, you can compute `TableCode & ~7`.
- `QuotaProcess`: A pointer to the process to which the handle table belongs. It can come in handy if you find handle tables using the pool-scanning approach described in Chapter 5 rather than enumerating processes and following their `ObjectTable` pointer.
- `HandleTableList`: A linked list of process handle tables in kernel memory. You can use it to locate other handle tables—potentially even those for processes that have been unlinked from the process list.
- `HandleCount`: The total number of handle table entries that are currently in use by the process. This field was removed starting in Windows 8 and Server 2012.

The key points for `_HANDLE_TABLE_ENTRY` are these:

- `Object`: This member points to the `_OBJECT_HEADER` of the corresponding object. The `_EX_FAST_REF` is a special data type that combines reference count information into the least significant bits of the pointer.
- `GrantedAccess`: A bit mask that specifies the granted access rights (read, write, delete, synchronize, etc.) that the owning process has obtained for the object.

---

## Enumerating Handles in Memory

The Volatility `handles` plugin generates output by walking the handle table data structures. Using this plugin without any options generates the most verbose output: all handles for all object types in all processes. So you should learn about a few filtering options:

- **Filter by process ID**: You can pass one or more (comma-separated) process IDs to the `-p/--pid` option.
- **Filter by process offset**: You can supply the physical offset of an `_EPROCESS` structure to the `-o/--offset` option.
- **Filter by object type**: If you’re interested in only a particular type of object, such as files or registry keys, you can specify the appropriate name(s) to the `-t/--object-type` option. See Chapter 5 or enter `!object \ObjectTypes` into Windbg to see the full list of object types.
- **Filter by name**: Not all objects have names. Unnamed objects are obviously useless when searching for indicators by name, so one way to reduce noise is to use the `--silent` option, which suppresses handles to unnamed objects.

### Finding Zeus Indicators

The following output shows the first few handles from PID 632 (`winlogon.exe`). The memory dump is from an older 32-bit XP system infected with Zeus ([https://code.google.com/p/malwarecookbook/source/browse/trunk/17/1/zeus.vmem.zip](https://code.google.com/p/malwarecookbook/source/browse/trunk/17/1/zeus.vmem.zip)). However, it’s difficult to discern the infection because the process has about 550 open handles (truncated for brevity) to different types of objects—and you may not immediately recognize them all.

---

**NOTE**

Recipe 9-5 of the *Malware Analyst’s Cookbook* includes source code for a C program that compares handles across all processes on the system to examine the effects of code injection. The code is based on the same API (`NtQuerySystemInformation`) that most tools on the running system use to enumerate handles. The code is available here: [https://code.google.com/p/malwarecookbook/source/browse/trunk/9/5/HandleDiff-src.zip](https://code.google.com/p/malwarecookbook/source/browse/trunk/9/5/HandleDiff-src.zip).

---

```
$ python vol.py -f zeus.vmem --profile=WinXPSP3x86 -p 632 handles 
Volatility Foundation Volatility Framework 2.4
Offset(V)     Pid     Handle     Access Type             Details
---------- ------ ---------- ---------- ---------------- -------
0xc10e007e18    632        0x4    0xc00f003 KeyedEvent       CritSecOutOfMemoryEvent
0xe1533748    632        0x8        0x3 Directory        KnownDlls
0xe17289f0    632       0x10  0x20f003f Key              MACHINE
0xc15e033d28    632       0x14    0xf000f Directory        Windows
0xc17e028b10    632       0x18  0x21c00f001 Port             
0xe16defd8    632       0x1c    0xf001f Section          
0x80ffa9c8    632       0x20  0x21c00f003 Event            
0xe1571708    632       0x24    0x2000f Directory        BaseNamedObjects
0x80f618e8    632       0x28   0x1c00f003 Event            crypt32LogoffEvent
[snip]
```

As shown in the next example, you can limit your search to files and mutexes opened by PID 632 and ignore unnamed objects. Although you’ll still see about 150 items (in this particular memory dump), the well-known Zeus artifacts are much easier to identify. For example, you see the `user.ds` and `local.ds` files, which contain the configuration and stolen data. The `sdra64.exe` file in the `system32` directory is the initial Zeus installer.

```
$ python vol.py -f zeus.vmem --profile=WinXPSP3x86 -p 632 handles 
      -t File,Mutant 
      --silent
Volatility Foundation Volatility Framework 2.4
Offset(V)     Pid     Handle     Access Type             Details
---------- ------ ---------- ---------- ---------------- -------
[snip]
0x80ff7b90    632      0x104   0x120089 File             
\Device\HarddiskVolume1\WINDOWS\system32\lowsec\user.ds
 [snip]
0xff12bb40    632      0x644   0x120089 File             
\Device\HarddiskVolume1\WINDOWS\system32\sdra64.exe
0xff13a470    632      0x648   0x120089 File             
\Device\HarddiskVolume1\WINDOWS\system32\lowsec\local.ds
0xff1e6b10    632      0x6dc   0x120116 File             \Device\Tcp
0xff1e6a38    632      0x6e0   0x1200a0 File             \Device\Tcp
0xff206778    632      0x6e4   0x1200a0 File             \Device\Ip
0xff1c66e010    632      0x6e8   0x100003 File             \Device\Ip
0xff1c65e078    632      0x6ec   0x1200a0 File             \Device\Ip
0x80f5cd78    632      0x898   0x12019f File             
\Device\NamedPipe\_AVIRA_2109
0xff1e7dc0    632      0x8bc   0x1c00f001 Mutant           _AVIRA_2109
```

You also may notice several open handles to `\Device\Tcp` and `\Device\Ip`. These are obviously different from the handles to files prefixed with `\Device\HarddiskVolume1`. Specifically, `Tcp` and `Ip` are *not* files on the machine’s hard drive. This is described in Chapter 11, but what you’re essentially seeing are artifacts of network sockets that the process creates. Although sockets aren’t files, they support similar operations such as opening, reading, writing, and deleting. As a result, the same handle/descriptor subsystem can service both files and network sockets.

Along the same lines, named pipes are also represented as file objects. Thus, if malware creates a named pipe for interprocess communication or to redirect output of a backdoor command shell into a file, you can determine which processes are involved in that activity, provided that you know the name of the pipe it creates. In this case, it’s easy to identify because the name of the pipe that Zeus uses is the same as the standard mutex it creates to mark its presence on systems (`_AVIRA_`).

### Detecting Registry Persistence

Malware often leverages the registry for persistence. To write the pertinent values, the malicious process must first open a handle to the desired registry key. In the following example, you’ll see how obvious it is when malware chooses a well-known location (such as the Run key) and also suffers from a handle leak. You should not only recognize the registry key name but also the fact that you have numerous open handles to the same key. Here is how the output appears:

```
$ python vol.py -f laqma.mem --profile=WinXPSP3x86 handles
      --object-type=Key 
      --pid=1700
Volatility Foundation Volatility Framework 2.4
Offset(V)     Pid     Handle     Access Type             Details
---------- ------ ---------- ---------- ---------------- -------
0xe12b6cb0   1700       0x10  0x20f003f Key              MACHINE
0xe12ae0e8   1700       0x60    0xf003f Key              
MACHINE\SYSTEM\CONTROLSET001\SERVICES\WINSOCK2\PARAMETERS\PROTOCOL_CATALOG9
0xe17a1c08   1700       0x68    0xf003f Key              
MACHINE\SYSTEM\CONTROLSET001\SERVICES\WINSOCK2\PARAMETERS\NAMESPACE_CATALOG5
0xe12382d8   1700       0x88    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe1ee99a8   1700       0x90    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe1fc7f18   1700       0x94    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe161bfb8   1700       0x98    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe18bbaf8   1700       0x9c    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe12a0348   1700       0xa0    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe1307598   1700       0xa4    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe150fb88   1700       0xa8    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe12e38f0   1700       0xac    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
[snip]
```

The process has nearly 20 open handles to the `RUN` key (not all are shown). This is indicative of a bug in the code that fails to close its handles after opening them. In this case, most likely there’s a loop that executes periodically to ensure that the persistence values are still intact (just in case an antivirus product or administrator removed them). Of course, the artifacts won’t always be this obvious, and just because a handle to a key is open, that doesn’t mean the process added values. However, you can always confirm your suspicions by using the `printkey` plugin (see Chapter 10) to look at the actual data that the key contains:

```
$ python vol.py -f laqma.mem printkey -K "MICROSOFT\WINDOWS\CURRENTVERSION\RUN"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: Run (S)
Last updated: 2012-11-28 03:05:07 UTC+0000

Subkeys:

Values:
REG_SZ        BluetoothAuthenticationAgent : (S) rundll32.exe 
bthprops.cpl,,BluetoothAuthenticationAgent
REG_SZ        VMware User Process : (S) 
      "C:\Program Files\VMware\VMware Tools\vmtoolsd.exe" -n vmusr
REG_SZ        lanmanwrk.exe   : (S) 
      C:\WINDOWS\System32\lanmanwrk.exe
REG_SZ        KernelDrv.exe clean : (S) 
      C:\WINDOWS\system32\KernelDrv.exe clean
```

Based on their names, the final two entries seem suspicious—they cause `lanmanwrk.exe` and `KernelDrv.exe` to start automatically at each boot. This run key is one of the most common persistence locations, so you probably would have checked here anyway and found the suspicious entries. However, by using the method described here (via the `handles` plugin) you can attribute the entries back to the exact process that created them.

### Identifying Remote Mapped Drives

Many adversaries rely on commands such as `net view` and `net use` to explore the surrounding network and map remote drives. Obtaining read access to a company’s Server Message Block (SMB) file server or write access to various other workstations or servers in an enterprise can lead to successful lateral movement. However, in these cases, the machine that performs the reconnaissance has open network connections to the remote systems and indicators of the activity in the process handle tables.

The following example shows how an attacker navigates the network to mount two remote drives. The `Users` directory of a system named `WIN-464MMR8O7GF` is mounted on `P`, and the `C$` share of a system named `LH-7J277PJ9J85I` is mounted at `Q`. If left unprotected, the attacker could also mount the `ADMIN$` share in the same manner. Once the drives are mapped, the attacker changes the current working directory of his command shell into a specific user’s documents folder.

```
C:\Users\Jimmy>net view
Server Name            Remark
-------------------------------------------
\\JAN-DF663B3DBF1
\\LH-7J277PJ9J85I
\\WIN-464MMR8O7GF
The command completed successfully.

C:\Users\Jimmy>net use p: \\WIN-464MMR8O7GF\Users
The command completed successfully.

C:\Users\Jimmy>net use q: \\LH-7J277PJ9J85I\C$
The command completed successfully.

C:\Users\Jimmy>net use
New connections will be remembered.

Status       Local     Remote                    Network
---------------------------------------------------------------------------
OK           P:        \\WIN-464MMR8O7GF\Users   Microsoft Windows Network
OK           Q:        \\LH-7J277PJ9J85I\C$      Microsoft Windows Network
The command completed successfully.

C:\Users\Jimmy>cd Q:\Users\Sharm\Documents

Q:\Users\Sharm\Documents
```

The trick to finding evidence of remote mapped drives in memory is to look for file handles prefixed with `\Device\Mup` and `\Device\LanmanRedirector`. MUP, which stands for Multiple Universal Naming Convention (UNC) Provider, is a kernel-mode component that channels requests to access remote files using UNC names to the appropriate network redirector. In this case, `LanmanRedirector` handles the SMB protocol.

Here’s an example of how the output of the `handles` plugin looks on the first hop machine (the one the attacker initially accessed). The system was running a 64-bit version of Vista SP2.

```
$ python vol.py -f hop.mem --profile=VistaSP2x64 handles -t File | grep Mup
Volatility Foundation Volatility Framework 2.4
Offset(V)             Pid       Handle      Access Type     Details
------------------ ------ ------------ ----------- -------- -------
[snip]
0xfffffa8001345c80    752         0xfc     0x100000 File    
\Device\Mup\;P:000000000002210f\WIN-464MMR8O7GF\Users
0xfffffa8003f02050    752        0x200     0x100000 File    \Device\Mup
0xfffffa80042c9f20    752        0x204     0x100000 File    \Device\Mup
0xfffffa80042dc410    752        0x208     0x100000 File    \Device\Mup
0xfffffa800433cf20    752        0x244     0x100000 File    \Device\Mup
0xfffffa800429cb10    752        0x258     0x100000 File    \Device\Mup
0xfffffa800134b190    752        0x264     0x100000 File    
\Device\Mup\;Q:000000000002210f\LH-7J277PJ9J85I\C$
0xfffffa800132b450   1544          0x8     0x100020 File    
\Device\Mup\;Q:000000000002210f\LH-7J277PJ9J85I\C$\Users\Jimmy\Documents
```

You have several plain handles to `\Device\Mup` (they are normal). The few in bold are the ones you should find interesting because they actually display the local drive letter, the remote NetBIOS name, and the share or file system path name. There are also two different process IDs shown: 752 and 1544. In this case, 752 is the instance of `svchost.exe` that runs the `LanmanWorkstation` service; it creates and maintains client network connections to remote servers using the SMB protocol. PID 1544 is the `cmd.exe` shell, and it has a handle to `C$\Users\Jimmy\Documents` as a result of the attacker changing into that directory.

Another way to detect remote mapped shares, which can be used in combination with the `handles` method, is to inspect symbolic links via the `symlinkscan` plugin. These kernel objects can be used to associate a drive letter, such as `Q` or `P`, with the redirected path. For example, the output looks like this:

```
$ python vol.py -f hop.mem --profile=VistaSP2x64 symlinkscan 
Volatility Foundation Volatility Framework 2.4
Offset(P)            #Ptr   #Hnd Creation time                  From       To  
------------------ ------ ------ ------------------------------ ---------- --
0x0000000024b0c6c0      1      0 2014-02-25 21:41:12 UTC+0000   Q:              
\Device\LanmanRedirector\;Q:0...00002210f\LH-7J277PJ9J85I\C$
0x0000000026f4a800      1      0 2014-02-25 21:40:45 UTC+0000   P:              
\Device\LanmanRedirector\;P:0...02210f\WIN-464MMR8O7GF\Users
```

One of the advantages of combining your methodologies is that with `symlinkscan`, you also get the exact time when the remote share was mounted. By incorporating this into your timeline, or (better yet) extracting the attacker’s command history from `cmd.exe` (see Chapter 17), you can quickly answer a lot of questions about the activities performed on the victim system or network.

## Summary

The evidence you look for, and the order in which you look for it, varies from case to case. However, in our experience, viewing the processes is a reasonable starting point, because it gives you an idea of what type of applications are running. As you’re analyzing the process list, keep in mind that malware frequently hides by blending in with critical system processes or by unlinking a process from the kernel’s process list. If you cannot fully identify a process by its name, leverage handles to determine what operating system resources the process is accessing. Also consider whether the user account under which the process is running should be allowed to perform the actions. Once you gain experience with these types of investigations, you can build (or trade) indicator lists with other analysts in the community and essentially automate the procedure to save time in the future.
