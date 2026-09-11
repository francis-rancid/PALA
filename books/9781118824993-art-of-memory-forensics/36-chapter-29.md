# Chapter 29  
 Mac Memory Overview

This chapter introduces a wide range of topics related to Mac memory forensics, including process analysis, recovery of cached files from memory, finding historical artifacts, and locating and extracting loaded kernel extensions. This chapter also highlights the similarities between analyzing Mac and Linux systems. Because these systems are both based on UNIX, our goal with this chapter is to introduce Mac-specific data structures and constructs without repeating information found in the Linux chapters. This chapter concludes with utilities that you can use to conduct live forensics of Mac systems. If you are familiar with performing live analysis on Linux, there might be a bit of a learning curve because many of the normal tools and techniques are not directly applicable to Mac OS X.

## Mac versus Linux Analysis

In Part III of the book, which discusses Linux forensics analysis, we cover a number of Volatility plugins, data structures, algorithms, rootkit techniques, detection capabilities, and other essential topics for when you perform memory forensics on Linux systems. As you will learn, Mac and Linux share many similarities, including adherence to the Portable Operating System Interface (POSIX) standard, which greatly influences operating system design, as well as the use of libc, bash, and other libraries and applications that are the foundation of the respective operating systems. Due to the great number of similarities and overlapping codebases, many of the analysis techniques and plugins you used on Linux memory samples are applicable on Mac systems. To aid with this transition, many of the Mac plugins are named the same as their Linux counterparts—with the `linux` prefix changed to `mac`.

In addition to memory forensics, this chapter includes a comparison of live forensics approaches for Linux and Mac. Even though live forensics is not a replacement for full memory forensics, it does have its uses, such as when you cannot acquire physical memory or when you want to build a dataset to cross-reference with the output of your memory forensics framework. This cross-referencing serves two purposes:

- To verify that your memory forensics tool produces correct results
- To find artifacts that a rootkit might be hiding from the live system

Although memory forensics can certainly find evidence of rootkit activity within a memory dump, without training and experience it might not be clear what effects the rootkit activity would have on the live system. The results of a live forensics investigation can help to quickly fill this knowledge gap.

## Process Analysis

The ability to locate and analyze process structures is a fundamental component of memory forensics. Once you find a process of interest, you can examine it to uncover memory maps, opened file descriptors, active network connections, and more. In this section, you learn how to recover processes using a number of methods as well as how to recognize abnormal process relationships.

---

## **Analysis Objectives**

Your objectives are these:

- **Understand the Mach and BSD split related to processes**: As you learned in Chapter 28, the Mac design includes both the Mach and BSD layers. These layers divide responsibilities in a way that requires multiple data structures to track each process.
- **Locate processes using multiple sources**: Kernel rootkits have the capability to hide processes from one or more kernel sources to mislead system administrators and live tools. Through memory forensics, you can enumerate processes in many ways and cross-reference them to find processes that were hidden on the live system.
- **Understand common parent/child process relationships**: A strong indicator of compromise is when processes are spawned by the wrong parent process. On Windows, it can be a `cmd.exe` spawned by Adobe Reader, and on Linux it can be a `netcat` instance running under Firefox. In this section, you learn the normal relationships of processes on Mac to help spot such anomalies.

## **Data Structures**

The `proc` structure tracks processes throughout the Mach layer.

```
>>> dt("proc")
'proc' (1192 bytes)
0x0   : p_list                         ['__unnamed_17118569']
0x10  : p_pid                          ['int']
0x18  : task                           ['pointer', ['task']]
0x20  : p_pptr                         ['pointer', ['proc']]
<snip>
0x30  : p_uid                          ['unsigned int']
0x34  : p_gid                          ['unsigned int']
<snip>
0x80  : p_sibling                      ['__unnamed_17118939']
0x90  : p_children                     ['__unnamed_17118990']
<snip>
0xe0  : p_fd                           ['pointer', ['filedesc']]
<snip>
0x2a0 : p_argslen                      ['unsigned int']
0x2a4 : p_argc                         ['int']
<snip>
0x2d4 : p_comm                         ['String', {'length': 17}]
0x2e5 : p_name                         ['array', 33, ['char']]
<snip>
```

The `task` structure tracks tasks throughout the BSD layer.

```
>>> dt("task")
'task' (960 bytes)
<snip>
0x20  : map                            ['pointer', ['_vm_map']]
0x28  : tasks                          ['queue_entry']
<snip>
0x40  : threads                        ['queue_entry']
<snip>
0x2f0 : bsd_info                       ['pointer', ['void']]
<snip>
0x308 : all_image_info_addr            ['unsigned long long']
0x310 : all_image_info_size            ['unsigned long long']
```

## **Key Points**

The key points for the `proc` structure are these:

- `p_list`: The process' linkage into the global list of running processes.
- `p_pid`: The process ID (PID).
- `task`: A pointer to the associated `task` structure in the BSD layer for this process.
- `p_pptr`: A pointer to the process' parent process.
- `p_uid` and `p_gid`: The user and group ID that the process started as.
- p_sibling and `p_children`: The list of processes started by the same parent process and spawned by this process. They build the parent/child relationship of processes generated by the `mac_pstree` plugin.
- `p_fd`: The file descriptor table of the process.
- `p_argslen` and `p_argc`: The number and length of the process' command-line arguments. It is used by the `mac_psaux` plugin.
- `p_comm` and `p_name`: The ASCII and Unicode name of the process.

- The key points for the `task` structure are these:

- `map`: The list of memory mappings.
- `tasks`: The task’s linkage into the global list of active tasks.
- `threads`: The threads of this task.
- `bsd_info`: A back pointer to the owning process (struct `proc`) of this task.
- `all_image_info_addr` and `all_image_info_size`: Store the address and size of the information that `dyld` (the dynamic loader) keeps on libraries loaded into the process. Later in this chapter, you’ll use them to examine the shared cache map.

---

### Enumerating Processes

The `mac_psxview` plugin can enumerate processes from Mac memory samples using several methods. The following output shows an example:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 
     -f 10.8.3x64.vmem mac_psxview
Volatility Foundation Volatility Framework 2.4 
Offset(V)          Name        PID  pslist parents pidhash  pgroup sleads tasks
------------------ ----------- ---- -----  ------- -------  ------ ------ ------
0xfffffc80f029ada2d0 kernel_task    0 True   True    False    True   True   True
0xffffff803012ca60 launchd        1 True   True    True     True   True   True
0xffffff803012bd40 kextd         12 True   False   True     True   True   True
0xffffff803012b480 taskgated     13 True   False   True     True   True   True
0xffffff803012b020 notifyd       14 True   False   True     True   True   True
0xffffff803012a760 securityd     16 True   False   True     True   True   True
0xffffff803012a300 configd       17 True   False   True     True   True   True
<snip>
0xffffff803345b480 bash         204 True   True    True     True   False  True
0xffffff803109a480 sudo         209 True   True    True     True   False  True
0xffffff8031098a40 dtrace       210 True   False   True     True   False  True
0xffffff80329a4a60 launchd      213 True   True    True     True   True   True
0xffffff803345c1a0 distnoted    216 True   False   True     True   False  True
0xffffff803345a760 cfprefsd     217 True   False   True     True   False  True
0xffffff803345a300 login        218 True   True    True     True   True   True
```

For each process, the virtual offset, name, and PID are printed along with True/False indicators that tell you from which source(s) each process was found. The following list describes the enumeration methods:

- **pslist**: Enumerates processes by walking the active process list linked within each `proc` structure. This column is generated through the `mac_pslist` plugin.
- **parents**: Walks the process list using `mac_pslist` and records the parent process for each process. To see the parent and children process of each process, use the `mac_pstree` plugin.
- **pidhash:** Walks the `pidhashtbl` global hash table of processes. This column is generated through the `mac_pid_hash_table` plugin.
- **pgroup**: Walks the `pgrphashtbl` global hash table of processes. This column is generated through the `mac_pgrp_hash_table` plugin.
- **sleads**: Enumerates the session leader process of each session. The list of sessions is generated through the `mac_list_sessions` plugin.
- **tasks**: Walks the global list of `task` structures and records the back pointer contained in the `bsd_info` member.

The purpose of the `mac_psxview` plugin is to enumerate processes using many methods to uncover hidden ones. As we present in Chapter 30, it successfully finds the process-hiding techniques employed by all known Mac rootkits. While studying the output of this plugin, be aware that some processes have `False` indicators, even though they are not malicious or hidden. A few benign `False` indicators that you commonly see are the following:

- The `kernel_task` process never appears in the `pid_hash` column.
- Many processes are `False` in the `parents` column, which occurs because only processes that spawn other processes are parent processes. For example, applications, such as text editors, PDF readers, and chat clients rarely spawn other processes, and as such do not appear in the `parents` column.
- Many processes are `False` in the `sleads` column. It occurs because only processes that lead session groups, such as `login`, are in this list. **NOTE** Sessions are a UNIX (Linux, BSD) concept that allows grouping multiple processes into a higher-level set, similar to Job objects in Windows. The purpose of these process sets is to efficiently manage signal handling. Because you can have only one leader per session, many processes are absent from the session leaders list.

### Process Relationships

You can use the `mac_pstree` plugin to visualize the parent/child relationship between processes. The following output is `mac_pstree` against a clean 64–bit Mountain Lion system:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 
   -f 10.8.3x64.vmem mac_pstree
Volatility Foundation Volatility Framework 2.4 
Name                 Pid             Uid
kernel_task          0               0
.launchd             1               0
..launchd            213             89
...mdworker          227             89
...cfprefsd          217             89
...distnoted         216             89
..coresymbolicatio   211             0
..com.apple.audio.   203             202
<snip>
..launchd            133             501
...Terminal          199             501
....login            218             0
.....bash            219             501
......sudo           222             0
.......dtrace        223             0
<snip>
...iTunesHelper      175             501
...vmware-tools-dae  170             501
...assistantd        167             501
...CalendarAgent     166             501
```

In this plugin’s output, `kernel_task` is the parent of all processes, so it appears at the root of the tree. During startup, `kernel_task` spawns the first userland process, which is the initial `launchd` with a PID of 1. This `launchd` is the parent of all future userland processes. Soon after the master `launchd` process is started, another `launchd`, PID 213 in the sample output, is started to handle the system daemons. Later, after a user logs in, a new launch daemon (PID 133) is spawned to handle the session.

In this session, the user launched the `Terminal` app, which created the `login` script, which then started `bash`. Once inside `bash`, the user elevated to root through `sudo` and then ran `dtrace`. You can tell that the user moved from a non-root user to root because the bash process (PID 219) started with the user ID (UID) of 501 and then `dtrace` runs with UID 0 after being spawned by `sudo`.

## Address Space Mappings

The mappings within the address space of a process can provide immensely useful information during an investigation, including the recovery of a process' heap, stack, application binary, shared libraries, mapped files, and more. To recover this information and make use of it, you must first understand how it is stored in memory along with some Mac-specific constructs.

---

## **Analysis Objectives**

Your objectives are these:

- **Learn Mac’s algorithms for memory mappings**: Mac has several unique features relating to its process memory mappings, including the use of bundles, library caches, and submappings. You will learn about these and how to analyze them to produce full mappings of data within a process' address space.
- **Understand the dynamic loader’s shared cache**: The shared library cache is a memory loading mechanism that has no counterpart on Windows and Linux. To effectively analyze a process' memory mappings, not only must you consult the kernel’s data structures but you must also consult those of `dyld`.

## **Data Structures**

The `_vm_map` structure represents a memory mapping within a process' address space:

```
>>> dt("_vm_map")
'_vm_map' (240 bytes)
0x0   : lock                           ['_lck_rw_t_internal_']
0x10  : hdr                            ['vm_map_header']
0x50  : pmap                           ['pointer', ['pmap']]
<snip>
```

The `vm_map_header` structure holds the initial link of process mappings:

```
>>> dt("vm_map_header")
'vm_map_header' (64 bytes)
0x0   : links                          ['vm_map_links']
0x20  : nentries                       ['int']
```

The `vm_map_links` structure stores the starting and ending address of each mapping, along with pointers to the rest of the mappings:

```
>>> dt("vm_map_links")
'vm_map_links' (32 bytes)
0x0   : prev                           ['pointer', ['vm_map_entry']]
0x8   : next                           ['pointer', ['vm_map_entry']]
0x10  : start                          ['unsigned long long']
0x18  : end                            ['unsigned long long']
```

The `vm_map_entry` structure holds the information required to recover the file path and contents of file-backed mappings:

```
>>> dt("vm_map_entry")
'vm_map_entry' (80 bytes)
<snip>
0x38  : object                         ['vm_map_object']
0x40  : offset                         ['unsigned long long']
<snip>
```

The `vm_map_object` structure holds information about the mapping’s submap or the file it maps:

```
>>> dt("vm_map_object")
'vm_map_object' (8 bytes)
0x0   : sub_map                        ['pointer', ['_vm_map']]
0x0   : vm_object                      ['pointer', ['vm_object']]
```

## **Key Points**

The key points for the `_vm_map` structure are these:

- `hdr`: A pointer to the beginning of the virtual memory maps for the process.
- `pmap`: A pointer to the physical memory maps of the process. You can use them to find the physical pages backing the maps' virtual addresses.

The key points for the `vm_map_header` structure are these:

- `links`: The head of the list of memory maps
- `nentries`: The number of memory maps

The key points for the `vm_map_links` structure are these:

- `next`and `prev`: The forward and backward pointers to the other memory maps of the process
- `start` and `end`: The starting and ending virtual address of the mapping within the process' address space

The key points for the `vm_map_entry` structure are these:

- `object`: The backing object of the map for file-backed mappings. It holds the information necessary to recover the path of the mapped file as well as its contents.
- offset: The offset of the mapping within the mapped file. This offset is *page aligned*, which means that even if a particular data structure is in the middle of a page, the offset points to the beginning of the containing page. Mach-O files are normally compiled so that the header is followed by the code segments and data segments. The code mapping starts at offset 0 of the file because the beginning of the code section is within the same page as the header, but the data mapping is at a non-zero offset within the file.

The key points for the `vm_map_object`structure are these:

- `sub_map`: Information for submappings of a particular map. This concept is explained in detail later in this section.
- `vm_object`: The structure that holds information about the mapped file. This topic is explored in more detail later in this chapter when you recover files cached in memory.

---

### Listing and Recovering Mappings

The `mac_proc_maps` plugin can be used to list process memory mappings. Here’s an example of finding the PID of the `iTunesHelper` process and then listing its memory maps:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 mac_pslist
Volatility Foundation Volatility Framework 2.4 
Offset             Name         Pid  Uid Gid PGID Bits  DTB        
-----------------  ------------ ---  --- --- ---- ----  ---------- 
0xffffff800d939098 iTunesHelper 223  501 20  223  64BIT 0x44238000 
<snip>

$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
      mac_proc_maps -p 223
Name          Start           End           Perms Map Name
------------- ------------- --------------  ----- -----------
iTunesHelper  0x100000000     0x100024000    r-x  <snip>/iTunesHelper
iTunesHelper  0x100024000     0x100026000    rw-  <snip>/iTunesHelper
iTunesHelper  0x100026000     0x100028000    rw-
iTunesHelper  0x100028000     0x100030000    r--  <snip>/iTunesHelper
iTunesHelper  0x100030000     0x100031000    r--
iTunesHelper  0x100031000     0x100032000    r--
iTunesHelper  0x100032000     0x100033000    rw-
<snip>
iTunesHelper  0x7fff662ac000  0x7fff662c00e000 r-x  Macintosh HD/usr/lib/dyld
iTunesHelper  0x7fff662c00e000  0x7fff662c20e000 rw-  Macintosh HD/usr/lib/dyld
iTunesHelper  0x7fff662c20e000  0x7ffc66f031f000 rw-
iTunesHelper  0x7ffc66f031f000  0x7fff66333000 r--  Macintosh HD/usr/lib/dyld
iTunesHelper  0x7fff70000000  0x7fff70a40000 r--  sub_map
iTunesHelper  0x7fff70a40000  0x7fff70c00000 rw-  <snip>dyld_shared_cache_x86_64
iTunesHelper  0x7fff70c00000  0x7fff70e00000 rw-  <snip>dyld_shared_cache_x86_64
<snip>
iTunesHelper  0x7fff73200000  0x7fff73249000 rw-  <snip>dyld_shared_cache_x86_64
iTunesHelper  0x7fff73249000  0x7fff80000000 r--  sub_map
iTunesHelper  0x7fff80000000  0x7fffc0000000 r--  sub_map
iTunesHelper  0x7fffc0000000  0x7fffffe00000 r--  sub_map
<snip>
```

In the output, the plugin lists the target PID and process name along with the starting and ending address of each mapping, the permissions, and map name (if any). The last three lines list the region as `sub_map` instead of an actual file path, which occurs because Mac groups files that are shared between processes into a submap so that it can efficiently manage them with respect to memory consumption. You will see an example of a common submap when we discuss the `dyld` shared cache.

To recover memory mappings from a process, you can use the `mac_dump_maps` plugin. To control which mappings are written to disk you can filter by process using the `-p/--pid` flag and filter by the starting address of the map of interest using the `-s/--map-address` flag. In the following output, we extract the mapping of the text segment of the `iTunesHelper` application:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64
       mac_dump_maps -p 223 -s 0x100000000 -D dumpdir
Volatility Foundation Volatility Framework 2.4 
Task VM Start    VM End      Length  Path
---- ----------- ----------- ------- ---------------
 223 0x100000000 0x100024000 0x24000 dumpdir/task.223.0x100000000.dmp
```

The recovered mapping is written to the specified directory. After extraction, you can then scan the recovered memory mappings with AV signatures, Yara rules, and other scanners. Note that we dumped only a portion of the mapped executable (the text segment). In Chapter 30, you learn how to recover the mapped file as a Mach-O file that you can load into IDA Pro for static analysis.

### Dynamic Loader Shared Cache

If you examine the `mac_proc_maps` output in the previous section, you’ll notice three sub maps towards the end of the output. The largest of these, which starts at `0x7fff80000000` and is 1GB in size, corresponds to the dynamic loader’s shared cache that is mapped into each process. The dynamic loader uses this cache to contiguously load a number of core and commonly used shared libraries into each process on startup. Sharing this cache across processes saves a significant amount of physical memory pages. It also provides substantial performance gains versus mapping the libraries at every process startup.

However, submaps cause a problem with memory forensics because they do not correspond to files on disk, and as such, there is no information within the kernel to tell us which libraries are mapped inside of the 1GB space. For instance, if you search for `.dylib` files (shared libraries) in the previous list of process mappings, none appear:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
     mac_proc_maps -p 223 | grep dylib
Volatility Foundation Volatility Framework 2.4
$
```

Even though the `iTunesHelper` application is dynamically linked, none of the dynamic link libraries it uses appear in the kernel’s mapping data structures. This is because the submap is managed by `dyld`. As such, the only way to determine the contents of the 1GB range is to consult the data structures of `dyld` stored within process memory.

The previously discussed `all_image_info_addr` member of the `task` structure provides the address of where this `dyld` data structure begins. The data structure stored at this address is a `dyld_all_image_infos` structure whose `infoArray` array member points to an `infoArrayCount` number of `dyld_image_info` structures. Each `dyld_image_info` structure contains the load address of the corresponding Mach-O file and the full path to the file on disk for its mapped library. The `mac_dyld_maps` plugin can find and enumerate the dynamic loader’s set of executable mappings, as shown here:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
     mac_dyld_maps -p 223
Volatility Foundation Volatility Framework 2.4 
Pid      Name                 Start              Map Name
-------- -------------------- ------------------ --------
223      iTunesHelper         0x0000000100000000 <snip>/iTunesHelper
223      iTunesHelper         0x00007fff82aec000 
/System/Library/Frameworks/IOKit.framework/Versions/A/IOKit
223      iTunesHelper         0x00007fff83db2000 
/System/Library/Frameworks/Carbon.framework/Versions/A/Carbon
223      iTunesHelper         0x00007fff811fa000 
/System/Library/Frameworks/DiskArbitration.framework/Versions/A/DiskArbitration
223      iTunesHelper         0x00007fff89ecd000 
/System/Library/Frameworks/Foundation.framework/Versions/C/Foundation
223      iTunesHelper         0x00007fff88928000 /usr/lib/libobjc.A.dylib
223      iTunesHelper         0x00007fff867c5000 /usr/lib/libstdc++.6.dylib
223      iTunesHelper         0x00007fff82015000 /usr/lib/libSystem.B.dylib
<snip>
```

The plugin lists the starting address from which the executable is mapped along with its full path on disk. Even though `mac_proc_maps` cannot find any of these libraries, the unfiltered output of `mac_dyld_maps` produces information on several libraries loaded into the process. They are all part of the `dyld` cache. If you study the start address of each loaded library, you notice that they are all within the range of the previously shown 1GB submap.

Understanding the purpose of the `dyld` cache is crucial during Mac process memory analysis because it contains the memory addresses of all the core libraries. So in order to check for API hooks and code overwrites, you must use the `dyld` structures instead of looking at the kernel data structures. Similarly, if your memory forensics tool indicates that a hook was placed within the mapped region, all you would know from `mac_proc_maps` is that the hook is within an anonymous 1GB submap. If you cannot map the hook back to the actual library and function, this will greatly hinder your analysis capabilities.

## Networking Artifacts

The ability to recover information about network activity on the target system can provide a wealth of forensic evidence. Such data commonly includes command and control servers that malware contacts, the IP address that attackers use to connect to the compromised system, and the IP addresses of systems targeted during lateral movement.

---

## **Analysis Objectives**

Your objectives are these:

- **Associate network activity with a specific process**: In many investigations, the initial indicator of compromise is when you receive alerts about a system beaconing out to known bad IPs or an attacker using a compromised system for lateral movement. By utilizing these types of network indicators, which will include the source and destination IP addresses and ports, you can tie the malicious activity back to a specific process.
- **Gather network connections from multiple sources**: The OS X kernel provides several data structures that enable the enumeration of active network connections. Knowing the differences between the enumeration techniques can help you determine the origin of a connection (process or kernel driver).
- **Classify network activity**: Once you have gathered information about the network activity of a system, you then need to determine which, if any, connections were malicious. Thus, this section illustrates Mac-centric network activity that you will commonly see the operating system and its stock applications performing.

## **Data Structures**

Each network connection is represented by a struct `socket`:

```
>>> dt("socket")
'socket' (880 bytes)
0x0   : so_zone                        ['int']
0x4   : so_type                        ['short']
0x8   : so_options                     ['unsigned int']
0xc   : so_linger                      ['short']
0xe   : so_state                       ['short']
0x10  : so_pcb                         ['pointer', ['void']]
0x18  : so_proto                       ['pointer', ['protosw']]
```

The socket protocol and state are stored in a `protosw` structure:

```
>>> dt("protosw")
'protosw' (160 bytes)
0x0   : pr_type                        ['short']
0x4   : pr_domain                      ['pointer', ['domain']]
0xc   : pr_protocol                    ['short']
```

The source and destination IP address and port information for IPv4 and IPv6 sockets are stored in an `inpcb` structure:

```
>>> dt("inpcb")
'inpcb' (392 bytes)
<snip>
0x7c  : inp_dependfaddr                ['__unnamed_11443658']
0x8c  : inp_dependladdr                ['__unnamed_11443712']
<snip>
```

## **Key Points**

The key points for struct `socket` are these:

- `so_state`: The state of the connection (e.g., ESTABLISHED, LISTENING). It is currently applicable only to TCP because it is the only protocol analyzed that maintains state.
- `so_pcb`: An opaque pointer to the protocol control block for the socket. For IPv4 and IPv6 connections, it is of type `inpcb`.

The key points for struct `protosw` are these:

- `pr_domain`: The socket domain of the socket. For common sockets such as TCP and UDP, it is AF_INET.
- `pr_protocol`: The socket protocol, such as TCP, UDP, UNIX, or RAW.

The key points for struct `inpcb` are these:

- `inp_dependfaddr`: The foreign (remote) IP address and port of the connection
- `inp_dependladdr`: The local IP address and port of the connection

---

#### Process File Descriptors

The `mac_netstat` plugin can recover network connections on a per-process basis. It operates by inheriting from `mac_lsof` to receive each file descriptor of analyzed processes. It then filters for descriptors of type `DTYPE_SOCKET`. These descriptors are then converted to a `socket` structure and examined. The following shows `mac_netstat` on a clean 10.8.3 system:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
          mac_netstat
Volatility Foundation Volatility Framework 2.4 
Proto  Local IP     Local Port  Remote IP  Rem Port State       Process
------ ----------   ----------  ---------- -------- --------    ---------------
UDP    0.0.0.0             137  0.0.0.0          0              launchd/1
UDP    0.0.0.0             138  0.0.0.0          0              launchd/1
TCP    ::1                 631  ::               0  LISTEN      launchd/1
TCP    127.0.0.1           631  0.0.0.0          0  LISTEN      launchd/1
UDP    ::                    0  ::               0              configd/17
       ::                    0  ::               0              configd/17
UDP    0.0.0.0           58570  0.0.0.0          0              syslogd/19
UDP    0.0.0.0            5353  0.0.0.0          0              mDNSResponder/34
UDP    ::                 5353  ::               0              mDNSResponder/34
UDP    0.0.0.0           49603  0.0.0.0          0              mDNSResponder/34
UDP    ::                49603  ::               0              mDNSResponder/34
UDP    0.0.0.0           53921  0.0.0.0          0              mDNSResponder/34
UDP    0.0.0.0             123 0. 0.0.0          0              ntpd/51
UDP    ::                  123 ::                0              ntpd/51
UDP    fe80:1::1           123 ::                0              ntpd/51
UDP    127.0.0.1           123 0.0.0.0           0              ntpd/51
UDP    ::1                 123 ::                0              ntpd/51
UDP    192.168.55.230      123 0.0.0.0           0              ntpd/51
UDP    0.0.0.0             138 0.0.0.0           0              netbiosd/65
UDP    0.0.0.0             137 0.0.0.0           0              netbiosd/65
TCP    192.168.55.230    49156 17.171.27.65    443  ESTABLISHED apsd/84
UDP    0.0.0.0               0 0.0.0.0           0              locationd/185
```

In addition to the connection information, `mac_netstat` also tells you which process started the network activity.

#### Networking Subsystem

Besides the per-process `socket` structures, the kernel also keeps records that track currently allocated network structures for each socket type (TCP, UDP, IP, RAW). The information stored for these protocols includes a parallel linked list and hash table of `inpcb` structures. The `mac_network_conns` plugin enumerates these data structures in order to recover all active network connections.

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
    mac_network_conns
Volatility Foundation Volatility Framework 2.4 
Offset (V)         Proto Local IP        Local Port Remote IP   Port State
------------------ ----- --------------- ---------- ------------ --- -------
0xffffff80305b6468 TCP   192.168.55.230  49156      17.171.27.65 443 ESTABLISHED
0xffffff80305b6488 TCP   127.0.0.1       631        0.0.0.0        0 LISTEN
0xffffff80305b1ba0 TCP   0.0.0.1         631        0.0.0.0        0 LISTEN
0xffffff80305b6bc0 TCP   0.0.0.1         631        0.0.0.0        0 LISTEN
0xffffff803021eae0 UDP   254.44.43.52    123        0.0.0.0        0
0xffffff803021f8c8 UDP   192.168.55.230  123        0.0.0.0        0
<snip>
```

#### Cross-Reference Advantage

After seeing `mac_netstat` and `mac_network_conns`, you might wonder why both are needed. They are necessary because each provides a different context. For instance, `mac_netstat` can report the process that started a connection, but `mac_network_conns` cannot. However, `mac_network_conns` is advantageous because it can find all network connections regardless of what code initiated them. So if a malicious kernel extension receives or sends network data, `mac_network_conns` can find the connection, whereas `mac_netstat` cannot.

#### Classifying Network Connections

Both of the previous plugin outputs were generated from clean systems, and there were a number of sockets across several processes. When performing memory forensics, you should be familiar with what network sockets are *normally* active so that you can more easily spot malicious ones. [Table 29-1](#table29-1) categorizes listening sockets that you will likely encounter when you analyze Mac systems. Note that this table lists only Mac-specific data, not information that you find across operating systems such as FTP servers listening on TCP port 21.

*[Table 29-1](#tableanchor29-1): Common Mac Network Sockets*

|  |  |  |  |
| --- | --- | --- | --- |
| **Application** | **Protocol** | **Port** | **Purpose** |
| launchd (PID 1) | UDP | 137 | NetBIOS. |
| launchd (PID 1) | UDP | 138 | NetBIOS. |
| launchd (PID 1) | TCP | 631 | Printing (CUPS). This socket should be in the LISTENING state. |
| apsd | TCP | 5223 | Apple’s push notification service daemon. This connection is normal if the remote IP is within Apple’s IP range. |
| netbiosd | UDP | 137 | NetBIOS. |
| netbiosd | UDP | 138 | NetBIOS. |
| mDNSResponder | UDP | 5353 | 5353 is the main mDNSResponder port. |
| mDNSResponder | UDP | 40000-60000 | mDNS will listen on a range of very high network ports for connections. |

[Table 29-1](#table29-1) lists the most commonly seen daemons and processes that perform network activity on default Mac systems. When you encounter network activity that the table does not categorize, you have a few other options to try and classify the connections. For example, `syslogd` and `ntpd` are not listed because they’re not specific to Mac. However, they’re standard on Linux/BSD. If you don’t recognize other services, consider the local and remote ports being used, and the reputation of the remote IPs (if any); then analyze the other activity being performed by the process or kernel module (its open file descriptors, loaded libraries, etc.).

## SLAB Allocator

This chapter has so far focused on artifacts related to processes, their memory mappings, and their interactions with the system and network. The next several sections focus on artifacts related to the kernel and its core data structures. We begin this exploration by studying the kernel’s main memory allocator: the SLAB allocator.

The SLAB allocator is used to allocate and deallocate memory structures that are frequently created and discarded. Examples of such structures include those related to process and thread creation, file system interaction, network activity, and interprocess communication. An interesting attribute of the SLAB allocator is that it keeps track of previously freed objects to quickly reuse them. Thus, during memory forensics examinations, you can leverage these freed entries within the zones to provide historical context of system activity.

---

## **Data Structures**

Each structure backed by a SLAB cache is represented by a `zone`:

```
>>> dt("zone")
'zone' (592 bytes)
0x0   : count                          ['int']
0x8   : free_elements                  ['unsigned long']
<snip>
0x198 : elem_size                      ['unsigned long']
0x1a0 : alloc_size                     ['unsigned long']
0x1a8 : sum_count                      ['unsigned long long']
<snip>
0x218 : zone_name                      ['pointer', ['String', {'length': 256}]]
<snip>
```

Free elements of a zone are represented by a `zone_free_element` structure:

```
>>> dt("zone_free_element")
'zone_free_element' (8 bytes)
0x0   : next                           ['pointer', ['zone_free_element']]
```

## **Key Points**

The key points for `zone` are these:

- `count`: The number of objects currently allocated from the zone.
- `free_elements`: A pointer to the first free element of the zone. They are of type `zone_free_element`.
- `elem_size`: The size of each object allocated from the zone.
- `sum_count`: The total number of objects allocated from the zone. Subtracting the `count` value from it gives the number of objects freed from the zone since its creation.
- `zone_name`: The name of the zone (proc, tasks, etc.).

The key points of the `zone_free_element` structure are these:

- `next`: A pointer to the next free element within the zone.

---

To illustrate the zones of a typical Mac system, the output of the `mac_list_zones` plugin is shown. In this plugin’s output, you can see the name of the zone along with the number of active and free elements:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
     mac_list_zones
Volatility Foundation Volatility Framework 2.4 
Name                           Active Count Free Count Element Size
------------------------------ ------------ ---------- ------------
zones                                   182          0          592
vm.objects                            18840      88522          224
vm.object.hash.entries                 5059        101           40
maps                                    108      56790          232
VM.map.entries                        36293     266241           80
Reserved.VM.map.entries                  72       3996           80
VM.map.copies                             0       1887           80
pmap                                     96        347          256
vm.pages                             511869      88151           72
tasks                                    90        138          928
threads                                 364       1641         1360
mbuf                                      0          0          256
socket                                  270        863          880
zombie                                    0        138          144
namei                                     0      18184         1024
proc                                     89        138         1120
udpcb                                    52        230          392
tcpcb                                     1          3          992
llinfo_arp                                3          1           56
<snip>
```

Many of these zones, such as `proc` (processes), `tasks` (task structures), `sockets` (socket objects), `udpcb` (UDP connections), `tcpcb` (TCP connections), and `llinfo_arp` (ARP data) should look familiar. Note that within the memory sample analyzed—which came from a system that had recently rebooted—there are already a number of freed objects of various types.

The following shows the output of `mac_dead_procs` against this memory sample. This plugin operates by finding the `proc` zone, enumerating its free elements, casting each element as a `proc` structure, and then reporting the process information.

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 
   -f 10.8.3x64.vmem mac_dead_procs
Volatility Foundation Volatility Framework 2.4 
Offset             Name    Pid   Uid  Gid  PGID    Bits DTB  Start Time
------------------ ------- ----  ---- ---- ----    ---- ---  ------------------
0xffffff803012b8e0 backupd  30      -  -   -55...11  -   -   2013-05-14 15:19:53 
0xffffff803012b8e0 backupd  30      -  -   -55...11  -   -   2013-05-14 15:19:53 
0xfffffc80f030f5e000 mdworker 226     -  -   -55...11  -   -   2013-05-14 15:21:04 
0xffffff8033459a40 path     221     -  -   -55...11  -   -   2013-05-14 15:20:22 
<snip>
0xffffff80334588c0 ???????? -55...37 - -  -55...37   -   -
```

Several of the members—such as UID, GID, Bits, and DTB—print “-” because they point to memory ranges that were freed since the process exited. However, the intact information is still quite useful because it includes the name, PID, and start time of each recovered historical process.

Volatility also provides the `mac_dead_sockets` and `mac_dead_vnodes` plugins that can recover historical information of the respective types. Due to the Volatility `zone` API, if historical records of other types are ever needed during an investigation, you can easily write a plugin to find and report the artifacts.

## Recovering File Systems from Memory

As you saw when you learned about the `dumpfiles` plugin for Windows (Chapter 16) and the `linux_recover_filesystem`plugin (Chapter 24) for Linux, recovering files from the in-memory file cache can greatly aid analysis. This section covers the plugins that enable examination and extraction of cached files stored on Mac systems. In this section, you learn how Mac caches files in memory and the strategies you can use to focus on key files. In Chapter 31, you revisit the plugins from this section to recover a number of files that are useful to track user activity on Mac systems.

---

## **Analysis Objectives**

Your objectives are these:

- **Recover files from memory**: There are many investigative scenarios in which you receive only a memory sample, not the corresponding disk image. By analyzing the operating system’s cache of files in memory, you can recover full files without the need for disk images.
- **Use file metadata to create timelines**: Besides raw file contents, the kernel also keeps track of metadata related to files, such as MAC times, owner, and permissions. This metadata from memory can help you create timelines and perform keyword searching (on the filenames).

## **Data Structures**

Each file in memory is represented by a `vnode` structure:

```
>>> dt("vnode")
'vnode' (248 bytes)
0x0   : v_lock                         ['__unnamed_14874329']
0x10  : v_freelist                     ['__unnamed_14887332']
0x20  : v_mntvnodes                    ['__unnamed_14887386']
<snip>
0x70  : v_un                           ['__unnamed_14887713']
<snip>
0xb0  : v_name                         ['pointer', ['String', {'length': 256}]]
0xb8  : v_parent                       ['pointer', ['vnode']]
```

The contents of the file are stored differently depending on the type of file:

```
>>> dt("__unnamed_14887713")
'__unnamed_14887713' (8 bytes)
0x0   : vu_fifoinfo                    ['pointer', ['fifoinfo']]
0x0   : vu_mountedhere                 ['pointer', ['mount']]
0x0   : vu_socket                      ['pointer', ['socket']]
0x0   : vu_specinfo                    ['pointer', ['specinfo']]
0x0   : vu_ubcinfo                     ['pointer', ['ubc_info']]
```

The contents of regular files are stored using the `ubc_info` structure:

```
>>> dt("ubc_info")
'ubc_info' (72 bytes)
0x0   : ui_pager                       ['pointer', ['memory_object']]
0x8   : ui_control                     ['pointer', ['memory_object_control']]
<snip>
```

The `memory_object_control` structure controls all operations on the file’s contents:

```
>>> dt("memory_object_control")
'memory_object_control' (16 bytes)
0x0   : moc_ikot                       ['unsigned int']
0x4   : _pad                           ['unsigned int']
0x8   : moc_object                     ['pointer', ['vm_object']]
```

The `vm_object` structure holds the queue of physical pages backing a file:

```
>>> dt("vm_object")
'vm_object' (224 bytes)
0x0   : memq                           ['queue_entry']
<snip>
```

Each physical page is represented by a `vm_page` structure:

```
>>> dt("vm_page")
'vm_page' (72 bytes)
0x0   : pageq                          ['queue_entry']
0x10  : listq                          ['queue_entry']
0x20  : next                           ['pointer', ['vm_page']]
<snip>
0x3c  : phys_page                      ['unsigned int']
```

## **Key Points**

The key points for struct `vnode` are these:

- `v_mntvnodes`: Linkage into the list of other `vnode` structures that belong to a mount point.
- v_un: An anonymous union of types that you use based on the type of file represented by the `vnode`. You can use this union to represent regular files and directories, sockets, and IPC pipes.
- `v_name`: The name of the `vnode`.
- `v_parent`: The parent of this `vnode`. You can use this pointer to build the full path of the `vnode` on disk.

The key points of struct `ubc_info` are these:

- `ui_pager`: A reference to the pager that handles read/write operations on the `vnode` and its backing store
- `ui_control`: A reference to the structure that brokers operations on the file through IPC messages and contains the set of physical pages backing a file

The key points of `memory_object_control` are these:

- `moc_ikot`: The IPC message handler for the object
- `moc_object`: A reference to the `vm_object` structure that holds the queue of the file’s pages

The key points of `vm_page` are these:

- `next`: The next physical page in the list of the file’s pages.
- `phys_page`: The page frame number of the page in physical memory. You can find the contents of the page by multiplying this number by the page size (4096) and reading the data at that offset in physical memory.

---

Volatility provides two methods to recover files from memory for Mac OS X. The first method allows you to list all available files in the memory dump and then selectively recover individual files by using `mac_list_files` and `mac_dump_file`. Running `mac_list_files` produces the full path of each file in memory along with the virtual address of its `vnode` structure, as shown here:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
      mac_list_files
Volatility Foundation Volatility Framework 2.4 
Offset (V)         File Path
------------------ ---------
0xfffffc80f030f05e88 /Macintosh HD/private/etc/master.passwd
0xffffff8032811aa8 /Macintosh HD/Library/Keychains/apsd.keychain
0xffffff80393b5d10 /Macintosh HD/Library/Keychains/apsd.keychain
<snip>
```

Once a file of interest is found, which in this example is `/etc/master.passwd`, the virtual address can then be passed to `mac_dump_file`:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
       mac_dump_file  -q 0xfffffc80f030f05e88 -O master.passwd
Volatility Foundation Volatility Framework 2.4 
Wrote 4096 bytes to master.passwd from vnode at address fffffc80f030f05e88

$ head -20 master.passwd
##
# User Database
#
# Note that this file is consulted directly only when the system is running
# in single-user mode.  At other times this information is provided by
# Open Directory.
#
# See the opendirectoryd(8) man page for additional information about
# Open Directory.
##
nobody:*:-2:-2::0:0:Unprivileged User:/var/empty:/usr/bin/false
root:*:0:0::0:0:System Administrator:/var/root:/bin/sh
daemon:*:1:1::0:0:System Services:/var/root:/usr/bin/false
_uucp:*:4:4::0:0:Unix to Unix Copy Protocol:/var/spool/uucp:/usr/sbin/uucico
_taskgated:*:13:13::0:0:Task Gate Daemon:/var/empty:/usr/bin/false
_networkd:*:24:24::0:0:Network Services:/var/empty:/usr/bin/false
_installassistant:*:25:25::0:0:Install Assistant:/var/empty:/usr/bin/false
_lp:*:26:26::0:0:Printing Services:/var/spool/cups:/usr/bin/false
_postfix:*:27:27::0:0:Postfix Mail Server:/var/spool/postfix:/usr/bin/false
_scsd:*:31:31::0:0:Service Configuration Service:/var/empty:/usr/bin/false
```

We were able to recover the full contents of the user database directly from the memory dump.

The second method for recovering files from memory is through the `mac_recover_filesystem` plugin. This plugin operates the same as `linux_recover_filesystem` (which was discussed in Chapter 24) in that it recovers the full file system structure, including metadata, to the directory that the user specifies. Using the gathered metadata (MAC times, file owner and permission, etc.), you can create detailed timelines of file system activity. You can also use the recovered file system’s metadata to automate common tasks, such as gathering all the files of a specific user or looking for files that attackers and malware create and modify.

Internally the file system plugins operate by first finding the list of mount points and then walking the mount point’s `vnode` structures. Each mount point is represented by a `mount` structure, and you enumerate them in Volatility through the `mac_mount` plugin. The `mnt_vnodelist` member of `mount` holds a pointer to the first `vnode` of the file system, and the `v_mntnodes` member is then used to enumerate the rest of the files.

## Loaded Kernel Extensions

As you saw with Windows and Linux, a common technique utilized by malware is to load kernel modules (called extensions in Mac OS X). By implementing a rootkit as a kernel module, the malware author has complete control over the running kernel and the system’s hardware devices. In this section, you learn how to find and extract kernel modules, both benign and malicious. In Chapter 30, you see several Mac kernel rootkits that operate using kernel extensions. You also see how Volatility can detect both the kernel extensions and report on the actions that rootkits perform once they load.

---

## **Analysis Objectives**

Your objectives are these:

- **Find kernel modules in memory**: To locate kernel modules in memory, you must understand their related data structures and how they are loaded in memory.
- **Extract kernel modules from memory to disk**: The ability to extract kernel modules to disk creates a number of possibilities, such as static reverse engineering and signature scanning.
- **Understand the limitations of analysis of modules extracted from memory**: Modules extracted from memory can have limitations that frustrate analysis. The limitations include these: the module cannot be loaded on another system for dynamic analysis, pages of the module might not be not present in memory due to paging, and malware can tamper with analysis by overwriting parts of the file’s metadata.

## **Data Structures**

Each kernel module is represented by a `kmod_info` structure:

```
>>> dt("kmod_info")
'kmod_info' (196 bytes)
0x0   : next                           ['pointer', ['kmod_info']]
<snip>
0x10  : name                           ['String', {'length': 64}]
<snip>
0x9c  : address                        ['unsigned long']
0xa4  : size                           ['unsigned long']
0xac  : hdr_size                       ['unsigned long']
0xb4  : start                          ['pointer', ['void']]
0xbc  : stop                           ['pointer', ['void']]
```

The IOKit subsystem maintains its own set of loaded kernel modules, which are represented by the `OSKext` C++ class. Note that the Volatility Mac profile creation code renames C++ classes to `<name>_class` to avoid conflicts when there are C structures and C++ classes of the same name.

```
>>> dt("OSKext_class")
'OSKext_class' (120 bytes)
0x10  : infoDict                       ['pointer', ['OSDictionary_class']]
0x18  : bundleID                       ['pointer', ['OSSymbol_class']]
0x20  : path                           ['pointer', ['OSString_class']]
<snip>
0x48  : kmod_info                      ['pointer', ['kmod_info_class']]
<snip>
```

## **Key Points**

The key points for struct `kmod_info` are these:

- `next`: A pointer to the next kernel module in the list
- `name`: The name of the kernel module
- `address`: The load address of the module’s Mach-O file in memory
- `size`: The size of the Mach-O file
- `start`: Pointer to the initialization routine for the module
- `stop`: Pointer to the routine called when the module is unloaded

The key points for class `OSKext` are these:

- `path`: The full path of the module on disk. This information is not available solely from the `kmod_info` structure.
- `kmod_info`: A pointer to the module’s `kmod_info` structure

---

### Enumerating Kernel Modules

Mac stores kernel modules in two data structures. The first is the list of modules referenced in the `kmod` global variable. The first member of this list actually represents the last module loaded onto the system. By following the `next` pointer of each module, all kernel modules can be enumerated in the *reverse* order of loading, which is implemented in the `mac_lsmod` plugin:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 mac_lsmod
Volatility Foundation Volatility Framework 2.4 
Offset (V)         Module Address     Size       Refs   Version      Name
------------------ ------------------ -------- -------- ------------ ----
0xffffff7f85a2b538 0xffffff7f85a27000    20480    0     1.60         
           com.apple.driver.AudioAUUC
0xffffff7f85a2b538 0xffffff7f854c3000    28672    0     650.4.0               
           com.apple.driver.AppleUSBMergeNub
0xffffff7f854c6e18 0xffffff7f85790000   118784    0     4.2.0f6        
           com.apple.iokit.IOBluetoothHostControllerUSBTransport
0xffffff7f857a4a70 0xffffff7c85f045c000    36864    0     650.4.4         
           com.apple.iokit.IOUSBHIDDriver
0xffffff7f85461e80 0xffffff7c85f050b000    16384    0     4.2.1b2          
           com.apple.driver.AppleUSBCDC
0xffffff7c85f050e9e8 0xffffff7f85465000    28672    1     650.4.0        
           com.apple.driver.AppleUSBComposite
0xffffff7f85468488 0xffffff7c86f015a000    40960    0     0137.86.37    
          com.vmware.kext.vmhgfs

<snip>

0xffffff7f860fa898 0xffffff7f84eba000   167936    19    2.8          
          com.apple.iokit.IOPCIFamily
0xffffff7f84ed4144 0xffffff7c85f011e000    36864    16    1.4          
          com.apple.iokit.IOACPIFamily
0xffffff7f85121a20 0xffffff7f85385000   290816    2     1.0          
          com.apple.kec.corecrypto
0xffffff7f853c54e0 0xffffff7f851aa000    45056    0     1            
          com.apple.kec.pthread
0xffffff7f851b2270 0x0000000000000000        0    48    13.0.0       
          com.apple.kpi.unsupported
0xffffff800a9dc000 0x0000000000000000        0    34    13.0.0       
          com.apple.kpi.private
0xffffff800a9dc100 0x0000000000000000        0    73    13.0.0       
          com.apple.kpi.mach
0xffffff800a9dc200 0x0000000000000000        0    86    13.0.0       
          com.apple.kpi.libkern
0xffffff800a9dc300 0x0000000000000000        0    81    13.0.0       
          com.apple.kpi.iokit
0xffffff800a9dc400 0x0000000000000000        0    6     13.0.0       
          com.apple.kpi.dsep
0xffffff800a9dc500 0x0000000000000000        0    62    13.0.0       
          com.apple.kpi.bsd
```

The plugin reports the virtual address of the `kmod_info` structure and the module’s Mach-O header, the size of the module, the number of references, the version, and the module name. Note that the last seven modules have a base address and size of 0, and a prefix of “com.apple.kpi”. These are the “fake” modules set up by the kernel so the kernel programming interfaces (KPIs) can be referenced through normal APIs.

The second data structure that tracks loaded kernel modules is the `sLoadedKexts` array inside of IOKit. The first reference to this artifact was snare’s presentation at Kiwicon 2011 ([http://ho.ax/downloads/Defiling_Mac_OS_X_Kiwicon.pdf](http://ho.ax/downloads/Defiling_Mac_OS_X_Kiwicon.pdf)). This array is stored using the `OSArray` class, and each element is of type `OSKext`. The `mac_lsmod_iokit` plugin can enumerate this array. It reports the same information as `mac_lsmod`, except that it also includes the full pathname for the module. For example, it displays `/System/Library/Extensions/corecrypto.kext` rather than `com.apple.kec.corecrypto`.

### Recovering Modules from Memory

Once you find a kernel module of interest, you can then extract it to disk using the `mac_moddump` plugin. In the following invocation, the `AudioAUUC`.`kext` kernel extension shown in the `mac_lsmod` output is extracted:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
   mac_moddump -D dumpdir -b 0xffffff7f85a2b538
Volatility Foundation Volatility Framework 2.4 
Address            Size     Output Path
------------------ -------- -----------
0xffffff7f85a27000    20480 com.apple.driver.AudioAUUC.0xffffff7f85a2b538.kext

$ file dumpdir/com.apple.driver.AudioAUUC.0xffffff7f85a2b538.kext
dumpdir/com.apple.driver.AudioAUUC.0xffffff7f85a2b538.kext: 
Mach-O 64-bit kext bundle x86_64
```

The operation of this plugin is actually very simple. The `address` member of `kmod_info` points to the base address of the Mach-O file in kernel memory, and the `size` member is the size of the Mach-O file. All the plugin must do is read `size` of bytes starting at `address` and write this data to disk.

## Other Mac Plugins

Many Mac plugins mirror functionality already described in the Linux-related chapters. So to avoid redundancy, the following list just introduces and discusses these plugins at a basic level.

- `mac_psaux`: Recovers the command-line arguments of a process, which is useful to determine which configuration flags were passed to an application.
- `mac_lsof`: Lists the open file descriptors of a process. Its output is formatted exactly like `linux_lsof`, in which the file descriptor number is listed along with the full path on disk for each file. For complete information on the data structures involved with `mac_lsof` we suggest reading Andrew F. Hay’s masters thesis ([http://reverse.put.as/wp-content/uploads/2011/06/FORENSIC-MEMORY-ANALYSIS-FOR-APPLE-OS-X.pdf](http://reverse.put.as/wp-content/uploads/2011/06/FORENSIC-MEMORY-ANALYSIS-FOR-APPLE-OS-X.pdf)).
- `mac_mount`: Provides information similar to `linux_mount`. For each mounted file system, the physical device, mount point, and file system type is listed.
- `mac_list_sessions`**:** Lists each login session along with the user that started the session and the session leader process.
- `mac_ifconfig`: This plugin is similar to `linux_ifconfig` and it lists the name and IP address of each network interface.
- `mac_dmesg`: Dumps the kernel debug buffer. Its Linux counterpart is `linux_dmesg`.
- `mac_route`: Lists the kernel’s routing table like its Linux counterpart `linux_route`.
- `mac_arp`: Lists the kernel’s ARP cache like its Linux counterpart `linux_arp`.
- `mac_bash`: Recovers the commands entered into the bash shell.
- `mac_bash_hash`: Recovers the command alias hash table.
- `mac_bash_env`: Recovers the environment variables of the bash session.

## Mac Live Forensics

Now that you have learned how to recover a wide range of information through memory forensics, we want to show you how to recover similar information from the live system. However, before you perform live forensics, you should be aware of its advantages and disadvantages. The main disadvantages are the ease in which malware can subvert live forensics tools, the inability to recover historical data, and the potential to destroy evidence by running commands on the live system. Because system administrators and security tools often rely on the live system APIs to find malware infections, attackers make a concerted effort to filter data out of the operating system’s reporting channels. This makes it difficult to detect advanced malware on a running system.

Likewise, the inability of live forensics tools to recover historical data stems from the fact that it can find and report only information that the kernel currently tracks. Because the system no longer uses historical data, the kernel has no need to keep a reference to it. Memory forensics analyzes physical memory without reliance on the live system’s reporting APIs, so it has the capability to both find malware hiding from live systems as well as find historical artifacts.

Even with its limitations, live forensics is useful in several situations. For example, there are times when you cannot acquire a full physical memory dump. It can occur due to lack of privileges, anti-forensics, or inaccessible memory dumping tools. Also, live APIs can provide a user’s view of the system resources, which can provide valuable information for detection or context about attacker intent.

[Table 29-2](#table29-2) shows information that is commonly collected from live machines as well as the counterpart Linux approach. You can group these commands into a live triage script that you can then include in your on-site field kit.

*[**Table 29-2**](#tableanchor29-2): Live Forensics Commands and Uses*

|  |  |  |
| --- | --- | --- |
| **Mac Command** | **Linux Command** | **Remarks** |
| `ps -ef` | `ps aux` | Lists each process with its name, command-line arguments, privileges, start time, and so on. |
| `vmmap` | `cat /proc/<pid>/maps` | Lists the memory mappings (application, shared libraries, stack, heap, etc.) of each process. |
| `lsof` | `lsof ls /proc/<pid>/fd` | Lists the open file handles of each process. |
| `netstat -an lsof -i` | `netstat -pan` | Lists the active network connections of the system along with the starting process. Note: On Mac, `netstat` does not show the PID of the owning process. Instead, you must use `lsof` to get this information. Note: The `n` flag to `netstat` tells it not to resolve DNS names and to print only the IP address. This can be useful to avoid resolving malware-related domains because they can rapidly change, and the DNS request can also tip off malware authors to your investigation. |
| `netstat -nr` | `route` | Lists the kernel’s routing table. |
| `arp -a` | `arp -a` | Lists the ARP cache. |
| `kextstat` | `lsmod cat /proc/modules` | Lists the set of currently loaded kernel modules/extensions. |
| `mount` | `mount` | Lists mounted file systems and their backing devices. |
| `uname` | `uname` | Lists the version of the kernel installed. |
| `uptime` | `uptime` | Lists the time since the last reboot. |
| `w / who / last` | `w / who / last` | Lists currently logged-in users and the last time each user account logged in. |
| `ifconfig` | `ifconfig` | Lists the network interfaces attached to the system. |
| `dmesg` | `dmesg` | Prints the kernel debug buffer. |
| `N/A` | `cat /proc/iomem` | Lists the regions of physical memory known to the kernel. Currently not available to userland tools on Mac. |
| `sysctl -a` | `sysctl -a` | Lists all `sysctl` names with their current value. |
| `history` | `history` | Returns the commands read from the `.bash_history` file of the current user along with commands from the current session. By default, `bash` writes to the history file only upon a clean termination of the shell. This means commands entered into active sessions might not yet be stored on disk. |
| `hash` | `hash` | Lists the mapping of commands run by the user (e.g., `ls`) to the full file path on disk (e.g., `/bin/ls`) as well as the number of times each command was run. You can use it to find proof of executed applications as well as malicious tampering of the bash environment (see Chapter 21). |

## Summary

Mac OS X is similar to Linux in many ways, because the two operating systems share common roots. Thus, the analysis techniques you implement on Linux (which you learned in Part III of this book) are often available and effective on Mac OS X. In particular, you can analyze processes, network activity, file system evidence, loaded kernel extensions, dynamic shared cache, and the SLAB allocator’s historical zone records. However, there are also key differences that can be critical during an investigation of Mac OS X systems, such as artifacts left by IOKit. Combining this Mac-specific knowledge with the techniques you learned about Linux gives you a powerful set of capabilities to leverage during memory forensics investigations.
